use inference_engine_rust::Core;
use inference_engine_rust::executable_network::infer_request::InferRequest;
use ndarray::{ArrayD, IxDyn};

#[test]
fn create_executable_network() {
    let core = Core::new();
    let network = core.read_network("test_data/resnet-50.xml",
                    "test_data/resnet-50.bin");
    let _executable_network = core.load_network(network, "CPU");
}

#[test]
fn create_core_and_find_device() {
    let core = Core::new();
    let devices = core.get_available_devices();
    assert!(!devices.is_empty());
    assert_eq!("CPU", devices[0]);
}
#[test]
fn create_infer_request() {
    let core = Core::new();
    let network = core.read_network("test_data/resnet-50.xml",
                    "test_data/resnet-50.bin");
    let executable_network = core.load_network(network, "CPU");
    let infer_request: InferRequest = executable_network.create_infer_request();

    let input = infer_request.get_blob("data");
    assert_eq!(input.dim(), IxDyn(&[1, 3, 224, 224]));
}

#[test]
fn get_output_blob() {
    let core = Core::new();
    let network = core.read_network("test_data/resnet-50.xml",
                    "test_data/resnet-50.bin");
    let executable_network = core.load_network(network, "CPU");
    let infer_request: InferRequest = executable_network.create_infer_request();

    let output = infer_request.get_blob("prob");
    assert_eq!(output.dim(), IxDyn(&[1, 1000]));
}

#[test]
fn can_change_input_blob() {
    let core = Core::new();
    let network = core.read_network("test_data/resnet-50.xml",
                    "test_data/resnet-50.bin");
    let executable_network = core.load_network(network, "CPU");
    let infer_request: InferRequest = executable_network.create_infer_request();
    {
        let mut input = infer_request.get_blob("data");
        input.fill(1.0);
    }
    let input = infer_request.get_blob("data");

    assert_eq!(input[[0, 0, 0, 0]], 1.0);
}

#[test]
fn can_set_blob_for_input() {
    let core = Core::new();
    let network = core.read_network("test_data/resnet-50.xml",
                    "test_data/resnet-50.bin");
    let executable_network = core.load_network(network, "CPU");
    let infer_request: InferRequest = executable_network.create_infer_request();
    let input_image = ndarray_image::open_image("test_data/cat.png",
        ndarray_image::Colors::Bgr).unwrap();
    // FIXME: move repacking into a helper
    let dims = input_image.dim();
    let width = dims.0;
    let height = dims.1;
    let channels = dims.2;
    let mut input = ArrayD::<f32>::zeros(IxDyn(&[1, channels, height, width]));
    for c in 0..channels {
        for h in 0..height {
            for w in 0..width {
                input[[0, c, h, w]] = input_image[[h, w, c]] as f32;
            }
        }
    }

    let elem = input[[0, 0, 0, 0]];
    infer_request.set_blob("data", input);

    let input_from_get = infer_request.get_blob("data");

    assert_eq!(input_from_get[[0, 0, 0, 0]], elem);
}
#[test]
fn get_correct_inference_results() {
    let core = Core::new();
    let network = core.read_network("test_data/resnet-50.xml",
                    "test_data/resnet-50.bin");
    let executable_network = core.load_network(network, "CPU");
    let infer_request: InferRequest = executable_network.create_infer_request();

    let mut input = infer_request.get_blob("data");
    let input_image = ndarray_image::open_image("test_data/cat.png",
        ndarray_image::Colors::Bgr).unwrap();
    // FIXME: move repacking into a helper
    let dims = input_image.dim();
    let width = dims.0;
    let height = dims.1;
    let channels = dims.2;
    for c in 0..channels {
        for h in 0..height {
            for w in 0..width {
                input[[0, c, h, w]] = input_image[[h, w, c]] as f32;
            }
        }
    }

    infer_request.infer();
    let output = infer_request.get_blob("prob");
    // FIXME: rulinalg is used only for argmax
    let argmax = rulinalg::utils::argmax(output.into_slice().unwrap());
    assert_eq!(argmax.0, 283);
}

#[test]
fn get_correct_inference_result_with_set_blob_for_input() {
    let core = Core::new();
    let network = core.read_network("test_data/resnet-50.xml",
                    "test_data/resnet-50.bin");
    let executable_network = core.load_network(network, "CPU");
    let infer_request: InferRequest = executable_network.create_infer_request();
    let input_image = ndarray_image::open_image("test_data/cat.png",
        ndarray_image::Colors::Bgr).unwrap();
    // FIXME: move repacking into a helper
    let dims = input_image.dim();
    let width = dims.0;
    let height = dims.1;
    let channels = dims.2;
    let mut input = ArrayD::<f32>::zeros(IxDyn(&[1, channels, height, width]));
    // TODO: move to a helper
    for c in 0..channels {
        for h in 0..height {
            for w in 0..width {
                input[[0, c, h, w]] = input_image[[h, w, c]] as f32;
            }
        }
    }

    infer_request.set_blob("data", input);
    infer_request.infer();
    let output = infer_request.get_blob("prob");
    // FIXME: rulinalg is used only for argmax
    let argmax = rulinalg::utils::argmax(output.into_slice().unwrap());
    assert_eq!(argmax.0, 283);
}
