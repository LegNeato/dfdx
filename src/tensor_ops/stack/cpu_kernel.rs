use crate::{
    shapes::*,
    tensor::{Cpu, Tensor},
    unique_id::unique_id,
};

use std::vec::Vec;

impl<E: Dtype> super::StackKernel<E> for Cpu {
    fn forward<S: Shape, Num: Dim>(
        &self,
        num: Num,
        inp: &[Tensor<S, E, Self>],
    ) -> Result<Tensor<S::Larger, E, Self>, Self::Err>
    where
        S: super::AddDim<Num>,
    {
        debug_assert_eq!(inp.len(), num.size());

        // check that all the strides are the same
        let item_strides = inp[0].strides;
        for i in inp.iter() {
            assert_eq!(i.strides, item_strides);
        }
        let shape: S::Larger = inp[0].shape().add_dim(num);

        // build the new strides
        let mut strides = shape.strides();
        strides[0] = inp[0].data.len();
        for d in 1..<S::Larger as Shape>::NUM_DIMS {
            strides[d] = item_strides[d - 1];
        }

        // copy the data
        let numel = strides[0];
        let mut data: std::vec::Vec<E> = std::vec::Vec::with_capacity(numel);
        for i in inp {
            data.extend_from_slice(i.data.as_ref());
        }

        Ok(Tensor {
            id: unique_id(),
            data: std::sync::Arc::new(data),
            shape,
            strides,
            device: self.clone(),
            tape: Default::default(),
        })
    }
    fn backward(
        &self,
        mut grad_inp: Vec<&mut Self::Vec<E>>,
        grad_out: &Self::Vec<E>,
    ) -> Result<(), Self::Err> {
        let mut offset = 0;
        for item in grad_inp.drain(..) {
            for gi in item.iter_mut() {
                *gi += grad_out[offset];
                offset += 1;
            }
        }
        Ok(())
    }
}
