//! Intro to dfdx::gradients and tapes

use dfdx::{
    gradients::{Gradients, NoneTape, OwnedTape},
    shapes::{Rank0, Rank2},
    tensor::{AsArray, SampleTensor, Tensor},
    tensor_ops::{Backward, MeanTo, TryMatMul},
};

#[cfg(not(feature = "cuda"))]
type Device = dfdx::tensor::Cpu;

#[cfg(feature = "cuda")]
type Device = dfdx::tensor::Cuda;

fn main() {
    let dev = Device::default();

    // tensors are actually generic over a fourth field: the tape
    // by default tensors are created with a `NoneTape`, which
    // means they don't currently **own** a tape.
    let weight: Tensor<Rank2<4, 2>, f32, _, NoneTape> = dev.sample_normal();
    let a: Tensor<Rank2<3, 4>, f32, _, NoneTape> = dev.sample_normal();

    // the first step to tracing is to call .trace()
    // this sticks a gradient tape into the input tensor!
    // NOTE: the tape has changed from a `NoneTape` to an `OwnedTape`.
    let b: Tensor<Rank2<3, 4>, _, _, OwnedTape<f32, Device>> = a.trace();

    // the tape will **automatically** be moved around as you perform ops
    // ie. the tapes on inputs to operations are moved to the output
    // of the operation.
    let c: Tensor<Rank2<3, 2>, _, _, OwnedTape<f32, Device>> = b.matmul(weight.clone());
    let d: Tensor<Rank2<3, 2>, _, _, OwnedTape<f32, Device>> = c.sin();
    let e: Tensor<Rank0, _, _, OwnedTape<f32, Device>> = d.mean();

    // finally you can use .backward() to extract the gradients!
    // NOTE: that this method is only available on tensors that **own**
    //       the tape!
    let gradients: Gradients<f32, Device> = e.backward();

    // now you can extract gradients for specific tensors
    // by querying with them
    let weight_grad: [[f32; 2]; 4] = gradients.get(&weight).array();
    dbg!(weight_grad);

    let a_grad: [[f32; 4]; 3] = gradients.get(&a).array();
    dbg!(a_grad);
}
