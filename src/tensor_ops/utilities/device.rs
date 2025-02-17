use super::super::ops::{BinaryKernel, UnaryKernel};
use crate::{
    shapes::Dtype,
    tensor::{CopySlice, DeviceStorage},
};

/// A [DeviceStorage] that requires all the tensor ops implementations
pub trait Device<E: Dtype>:
    DeviceStorage
    + CopySlice<E>
    + crate::tensor::TensorFromVec<E>

    + crate::tensor_ops::stack::StackKernel<E>

    // allocation
    + crate::tensor::ZerosTensor<E>
    + crate::tensor::OnesTensor<E>
    + crate::tensor::SampleTensor<E>
    + crate::tensor::OneFillStorage<E>
    + crate::tensor::ZeroFillStorage<E>

    // broadcast & reduces
    + super::super::broadcast_to::BroadcastKernel<E>
    + super::super::sum_to::SumKernel<E>
    + super::super::max_to::MaxReduceKernel<E>
    + super::super::min_to::MinReduceKernel<E>
    + super::super::permute_to::PermuteKernel<E>
    + super::super::reshape_to::ReshapeKernel<E>

    // indexing
    + super::super::select_and_gather::ReplaceDimKernel<E>
    + super::super::select_and_gather::RemoveDimKernel<E>
    + super::super::choose::ChooseKernel<E>

    // matmuls
    + super::super::matmul::VecMatKernel<E>
    + super::super::matmul::MatMatKernel<E>
    + super::super::matmul::VecVecKernel<E>
    + super::super::matmul::MatMatBrKernel<E>
    + super::super::matmul::MatMatBatch3Kernel<E>
    + super::super::matmul::MatMatBatch4Kernel<E>

    // scalar arithmetic
    + UnaryKernel<super::super::add::ScalarAddKernelOp<E>, E>
    + UnaryKernel<super::super::sub::ScalarSubKernelOp<E>, E>
    + UnaryKernel<super::super::mul::ScalarMulKernelOp<E>, E>
    + UnaryKernel<super::super::div::ScalarDivKernelOp<E>, E>

    // binary arithmetic
    + BinaryKernel<super::super::add::BinaryAddKernelOp, E>
    + BinaryKernel<super::super::sub::BinarySubKernelOp, E>
    + BinaryKernel<super::super::mul::BinaryMulKernelOp, E>
    + BinaryKernel<super::super::div::BinaryDivKernelOp, E>

    // boolean operations
    + super::super::boolean::BooleanKernel

    // unary
    + UnaryKernel<super::super::abs::AbsKernelOp, E>
    + UnaryKernel<super::super::clamp::ClampKernelOp<E>, E>
    + UnaryKernel<super::super::cos::CosKernelOp, E>
    + super::super::dropout::DropoutKernel<E>
    + UnaryKernel<super::super::exp::ExpKernelOp, E>
    + UnaryKernel<super::super::ln::LnKernelOp, E>
    + UnaryKernel<super::super::nans_to::NansToKernelOp<E>, E>
    + UnaryKernel<super::super::negate::NegateKernelOp, E>
    + UnaryKernel<super::super::relu::ReLUKernelOp, E>
    + UnaryKernel<super::super::gelu::GeLUKernelOp, E>
    + UnaryKernel<super::super::sigmoid::SigmoidKernelOp, E>
    + UnaryKernel<super::super::sin::SinKernelOp, E>
    + UnaryKernel<super::super::sqrt::SqrtKernelOp, E>
    + UnaryKernel<super::super::square::SquareKernelOp, E>
    + UnaryKernel<super::super::tanh::TanhKernelOp, E>
    + UnaryKernel<super::super::pow::PowfKernelOp<E>, E>
    + UnaryKernel<super::super::pow::PowiKernelOp, E>

    // binary
    + BinaryKernel<super::super::bce::BCEKernelOp, E>
    + BinaryKernel<super::super::huber_error::HuberErrorKernelOp<E>, E>
    + BinaryKernel<super::super::maximum::MaximumKernelOp, E>
    + BinaryKernel<super::super::minimum::MinimumKernelOp, E>
    + crate::tensor_ops::axpy::AxpyKernel<E>
{
}

impl Device<f32> for crate::tensor::Cpu {}
impl Device<f64> for crate::tensor::Cpu {}

#[cfg(feature = "cuda")]
impl Device<f32> for crate::tensor::Cuda {}

#[cfg(feature = "cuda")]
impl Device<f64> for crate::tensor::Cuda {}
