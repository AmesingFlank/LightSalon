use std::sync::Arc;

use salon_core::{
    editor::Editor,
    runtime::{Runtime, Toolbox},
    services::services::Services,
    session::Session,
};

use super::{image_comparer, ImageComparer};

pub struct TestContext {
    pub session: Session,
    pub image_comparer: ImageComparer,
}

impl TestContext {
    pub fn new() -> Self {
        let runtime = make_test_runtime();
        let image_comparer = ImageComparer::new(runtime.clone());
        TestContext {
            session: Session::new(runtime),
            image_comparer,
        }
    }
}

fn make_test_runtime() -> Arc<Runtime> {
    let instance = wgpu::Instance::default();

    let adapter = Arc::new(futures::executor::block_on(async move {
        instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("failed to request adapter")
    }));

    println!("{:#?}", adapter.get_info());

    let adapter_clone = adapter.clone();
    let (device, queue) = futures::executor::block_on(async move {
        adapter_clone
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: get_wgpu_limits_for_testing(),
                },
                None,
            )
            .await
            .expect("failed to request device")
    });

    Arc::new(Runtime::new(adapter, Arc::new(device), Arc::new(queue)))
}

fn get_wgpu_limits_for_testing() -> wgpu::Limits {
    wgpu::Limits {
        max_storage_buffer_binding_size: 134217728, // limits imposed by github actions machines
        ..Runtime::get_required_wgpu_limits()
    }
}
