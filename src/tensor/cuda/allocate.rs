#![allow(clippy::needless_range_loop)]

use crate::{
    shapes::*,
    tensor::{
        cpu::{Cpu, CpuError},
        storage_traits::*,
        Tensor,
    },
    unique_id::unique_id,
};

use super::{Cuda, CudaError};

use cudarc::driver::{CudaSlice, DeviceSlice};
use rand::Rng;
use std::{sync::Arc, vec::Vec};

impl Cuda {
    fn tensor_from_host_buf<S: Shape, E: Unit>(
        &self,
        shape: S,
        buf: Vec<E>,
    ) -> Result<Tensor<S, E, Self>, CudaError> {
        Ok(self.build_tensor(shape, shape.strides(), self.dev.htod_copy(buf)?))
    }

    pub(crate) fn build_tensor<S: Shape, E: Unit>(
        &self,
        shape: S,
        strides: S::Concrete,
        slice: CudaSlice<E>,
    ) -> Tensor<S, E, Self> {
        Tensor {
            id: unique_id(),
            data: Arc::new(slice),
            shape,
            strides,
            device: self.clone(),
            tape: Default::default(),
        }
    }
}

impl<E: Unit> ZerosTensor<E> for Cuda {
    fn try_zeros_like<S: HasShape>(&self, src: &S) -> Result<Tensor<S::Shape, E, Self>, Self::Err> {
        let shape = *src.shape();
        let strides = shape.strides();
        let data = self.dev.alloc_zeros(shape.num_elements())?;
        Ok(self.build_tensor(shape, strides, data))
    }
}

impl<E: Unit> ZeroFillStorage<E> for Cuda {
    fn try_fill_with_zeros(&self, storage: &mut Self::Vec<E>) -> Result<(), Self::Err> {
        self.dev.memset_zeros(storage)?;
        Ok(())
    }
}

impl<E: Unit> OnesTensor<E> for Cuda
where
    Cpu: OnesTensor<E>,
{
    fn try_ones_like<S: HasShape>(&self, src: &S) -> Result<Tensor<S::Shape, E, Self>, Self::Err> {
        let shape = *src.shape();
        let buf = std::vec![E::ONE; shape.num_elements()];
        self.tensor_from_host_buf(shape, buf)
    }
}

impl<E: Unit> OneFillStorage<E> for Cuda {
    fn try_fill_with_ones(&self, storage: &mut Self::Vec<E>) -> Result<(), Self::Err> {
        self.dev
            .htod_copy_into(std::vec![E::ONE; storage.len()], storage)?;
        Ok(())
    }
}

impl<E: Unit> SampleTensor<E> for Cuda
where
    Cpu: SampleTensor<E>,
{
    fn try_sample_like<S: HasShape, D: rand_distr::Distribution<E>>(
        &self,
        src: &S,
        distr: D,
    ) -> Result<Tensor<S::Shape, E, Self>, Self::Err> {
        let shape = *src.shape();
        let mut buf = Vec::with_capacity(shape.num_elements());
        {
            let mut rng = self.cpu.rng.lock().unwrap();
            buf.resize_with(shape.num_elements(), || rng.sample(&distr));
        }
        self.tensor_from_host_buf::<S::Shape, E>(shape, buf)
    }
    fn try_fill_with_distr<D: rand_distr::Distribution<E>>(
        &self,
        storage: &mut Self::Vec<E>,
        distr: D,
    ) -> Result<(), Self::Err> {
        let mut buf = Vec::with_capacity(storage.len());
        {
            let mut rng = self.cpu.rng.lock().unwrap();
            buf.resize_with(storage.len(), || rng.sample(&distr));
        }
        self.dev.htod_copy_into(buf, storage)?;
        Ok(())
    }
}

impl<E: Unit> CopySlice<E> for Cuda {
    fn copy_from<S: Shape, T>(dst: &mut Tensor<S, E, Self, T>, src: &[E]) {
        assert_eq!(
            dst.data.len(),
            src.len(),
            "Slices must have same number of elements as *physical* storage of tensors."
        );
        dst.device
            .dev
            .htod_sync_copy_into(src, Arc::make_mut(&mut dst.data))
            .unwrap();
    }
    fn copy_into<S: Shape, T>(src: &Tensor<S, E, Self, T>, dst: &mut [E]) {
        assert_eq!(
            src.data.len(),
            dst.len(),
            "Slices must have same number of elements as *physical* storage of tensors."
        );
        src.device
            .dev
            .dtoh_sync_copy_into(src.data.as_ref(), dst)
            .unwrap();
    }
}

impl<E: Unit> TensorFromVec<E> for Cuda {
    fn try_tensor_from_vec<S: Shape>(
        &self,
        src: Vec<E>,
        shape: S,
    ) -> Result<Tensor<S, E, Self>, Self::Err> {
        let num_elements = shape.num_elements();

        if src.len() != num_elements {
            Err(CudaError::Cpu(CpuError::WrongNumElements))
        } else {
            self.tensor_from_host_buf(shape, src)
        }
    }
}

impl<S: Shape, E: Unit> TensorToArray<S, E> for Cuda
where
    Cpu: TensorToArray<S, E>,
{
    type Array = <Cpu as TensorToArray<S, E>>::Array;
    fn tensor_to_array<T>(&self, tensor: &Tensor<S, E, Self, T>) -> Self::Array {
        let buf = tensor.data.try_clone().unwrap().try_into().unwrap();
        self.cpu
            .tensor_to_array::<crate::gradients::NoneTape>(&Tensor {
                id: tensor.id,
                data: Arc::new(buf),
                shape: tensor.shape,
                strides: tensor.strides,
                device: self.cpu.clone(),
                tape: Default::default(),
            })
    }
}
