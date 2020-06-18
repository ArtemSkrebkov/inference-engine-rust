extern crate inference_engine_rust;
use inference_engine_rust::Core;

use argparse::{ArgumentParser, Store};
use inference_engine_rust::utils;
use ndarray::{ArrayView, IxDyn};

fn main() {
    let mut device = String::new();
    let mut model_filename = String::new();
    let mut weights_filename = String::new();
    let mut image_filename = String::new();
    {
        let mut parser = ArgumentParser::new();
        parser.set_description("Classification single-inference sample");
        parser.refer(&mut device)
            .add_option(&["-d", "--device"], Store,
                          "Choose device to run").required();
        parser.refer(&mut model_filename)
            .add_option(&["-m", "--model"], Store,
                          "Choose model to run").required();
        // TODO: make weights optional
        parser.refer(&mut weights_filename)
            .add_option(&["-w", "--weights"], Store,
                          "Choose weights to use").required();
        parser.refer(&mut image_filename)
            .add_option(&["-i", "--image"], Store,
                          "Choose image to run").required();
 
        parser.parse_args_or_exit();
    }
    // TODO check validness

    let core = Core::new();
    let network = core.read_network(&model_filename,
                    &weights_filename);
    let executable_network = core.load_network(network, &device);
    let infer_request = executable_network.create_infer_request();
    let input_image = ndarray_image::open_image(image_filename,
        ndarray_image::Colors::Bgr).unwrap();
    // TODO: InferRequest supports only NCHW data for now,
    // that's why the convertion is required
    let input = utils::convert_layout_from_nhwc_to_nchw(input_image.view().into_dyn());
    // FIXME: input/output name can be extracted from IR. Need to make it available from API
    infer_request.set_blob("data", input);
    infer_request.infer();
    let output = infer_request.get_blob("prob");
    // TODO: make number optional argument
    let number = 5;
    // TODO: add labels as an optional parameter
    print_classification_results(output, number);
}

fn print_classification_results(data: ArrayView<f32, IxDyn>, k: usize) {
   let argmax = utils::argmax(data.to_slice().unwrap());
    for i in 0..k {
        println!("Top-{}: {}", i+1, argmax[i]);
    }
}
