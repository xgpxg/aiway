
#[macro_export]
macro_rules! export {
    ($plugin_type:ty) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn create_plugin() -> *mut dyn plugin::Plugin {
            let plugin = Box::new(<$plugin_type>::new());
            Box::into_raw(plugin)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn destroy_plugin(plugin: *mut dyn plugin::Plugin) {
            if !plugin.is_null() {
                unsafe {
                    let _ = Box::from_raw(plugin as *mut $plugin_type);
                }
            }
        }
    };
}
