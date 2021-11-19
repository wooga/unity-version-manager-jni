use std::ffi::c_void;
use std::ops::Deref;
use std::panic::catch_unwind;

use flexi_logger::{default_format, DeferredNow, LevelFilter, Logger, LogTarget, Record};
use flexi_logger::writers::LogWriter;
use jni::{JavaVM, JNIEnv};
use jni::objects::{GlobalRef, JClass, JMethodID, JObject, JStaticMethodID};
use jni::objects::JValue::Void;
use jni::signature::{JavaType, Primitive};
use jni::sys::{jint, JNI_ERR, JNI_GetCreatedJavaVMs, JNI_OK, JNI_VERSION_1_8};
use log::logger;
use parking_lot::Once;

static INIT: Once = Once::new();
const INVALID_JNI_VERSION: jint = 0;

static mut LOGGER_FINE: Option<JMethodID> = None;
static mut LOGGER_FINER: Option<JMethodID> = None;
static mut LOGGER_INFO: Option<JMethodID> = None;
static mut LOGGER_WARNING: Option<JMethodID> = None;
static mut LOGGER_SEVERE: Option<JMethodID> = None;
static mut LOGGER_GET_LOGGER: Option<JStaticMethodID> = None;

static mut FILE_GET_ABSOLUTE_PATH: Option<JMethodID> = None;

static mut LOGGER_CLASS: Option<GlobalRef> = None;
static mut FILE_CLASS: Option<GlobalRef> = None;
static mut INSTALLATION_CLASS: Option<GlobalRef> = None;
static mut COMPONENT_CLASS: Option<GlobalRef> = None;
static mut LOGGER_OBJ: Option<GlobalRef> = None;
static mut JNI_JAVA_VM: Option<JavaVM> = None;

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn JNI_OnLoad(vm: JavaVM, _reserved: *mut c_void) -> jint {
    let env = vm.get_env().expect("Cannot get reference to the JNIEnv");
    catch_unwind(|| {
        init_cache(&env);

        JNI_VERSION_1_8
    }).unwrap_or(INVALID_JNI_VERSION)
}

fn init_cache(env: &JNIEnv) {
    INIT.call_once(|| unsafe {
        cache_classes(env);
        cache_methods(env);
        cache_logger(env);
        crate::logging::setup_logger();
    });
}

unsafe fn cache_methods(env: &JNIEnv) {
    JNI_JAVA_VM = env.get_java_vm().ok();
    LOGGER_FINE = get_method_id(&env, "java/util/logging/Logger", "fine", "(Ljava/lang/String;)V");
    LOGGER_FINER = get_method_id(&env, "java/util/logging/Logger", "finer", "(Ljava/lang/String;)V");
    LOGGER_INFO = get_method_id(&env, "java/util/logging/Logger", "info", "(Ljava/lang/String;)V");
    LOGGER_WARNING = get_method_id(&env, "java/util/logging/Logger", "warning", "(Ljava/lang/String;)V");
    LOGGER_SEVERE = get_method_id(&env, "java/util/logging/Logger", "severe", "(Ljava/lang/String;)V");

    FILE_GET_ABSOLUTE_PATH = get_method_id(&env, "java/io/File", "getAbsolutePath", "()Ljava/lang/String");

    LOGGER_GET_LOGGER = get_static_method_id(&env, "java/util/logging/Logger", "getLogger", "(Ljava/lang/String;)Ljava/util/logging/Logger;");
}

unsafe fn cache_classes(env: &JNIEnv) {
    LOGGER_CLASS = get_class(&env, "java/util/logging/Logger");
    FILE_CLASS = get_class(&env, "java/io/File");
    INSTALLATION_CLASS = get_class(&env, "net/wooga/uvm/Installation");
    COMPONENT_CLASS = get_class(&env, "net/wooga/uvm/Component");
}

unsafe fn cache_logger(env: &JNIEnv) {
    let logger_name = env.new_string("net.wooga.uvm.UnityVersionManager.JNI").unwrap();
    let logger = env.call_static_method_unchecked(logging::get_logger_class_unchecked(), LOGGER_GET_LOGGER.unwrap(), JavaType::Object("java/util/logging/Logger".to_string()), &[logger_name.into()]).unwrap();

    LOGGER_OBJ = Some(env.new_global_ref(logger.l().unwrap()).unwrap());
}

pub mod io {
    use super::*;
    pub mod file {
        use super::*;

        pub fn get_file_class() -> JClass<'static> {
            unsafe { FILE_CLASS.as_ref().unwrap().as_obj().into() }
        }

        pub fn get_absolute_path_id() -> JMethodID<'static> {
            check_cache_initialized();
            unsafe { FILE_GET_ABSOLUTE_PATH.unwrap()}
        }

        pub fn get_absolute_path_type() -> JavaType {
            JavaType::Object("java/lang/String".to_string())
        }
    }
}

pub mod logging {
    use super::*;

    pub fn get_logger_id() -> JStaticMethodID<'static> {
        check_cache_initialized();
        unsafe { LOGGER_GET_LOGGER.unwrap() }
    }

    pub fn get_fine_id() -> JMethodID<'static> {
        check_cache_initialized();
        unsafe { LOGGER_FINE.unwrap() }
    }

    pub fn get_finer_id() -> JMethodID<'static> {
        check_cache_initialized();
        unsafe { LOGGER_FINER.unwrap() }
    }

    pub fn get_info_id() -> JMethodID<'static> {
        check_cache_initialized();
        unsafe { LOGGER_INFO.unwrap() }
    }

    pub fn get_warning_id() -> JMethodID<'static> {
        check_cache_initialized();
        unsafe { LOGGER_WARNING.unwrap() }
    }

    pub fn get_severe_id() -> JMethodID<'static> {
        check_cache_initialized();
        unsafe { LOGGER_SEVERE.unwrap() }
    }

    pub fn get_logger_class_unchecked() -> JClass<'static> {
        unsafe { LOGGER_CLASS.as_ref().unwrap().as_obj().into() }
    }

    pub fn get_logger_class() -> JClass<'static> {
        check_cache_initialized();
        get_logger_class_unchecked()
    }

    pub fn get_logger_obj() -> JObject<'static> {
        check_cache_initialized();
        unsafe { LOGGER_OBJ.as_ref().unwrap().as_obj().into() }
    }
}

pub mod system {
    use super::*;

    pub fn get_jvm() -> &'static JavaVM {
        check_cache_initialized();
        unsafe { JNI_JAVA_VM.as_ref().unwrap() }
    }
}

fn get_method_id(env: &JNIEnv, class: &str, name: &str, sig: &str) -> Option<JMethodID<'static>> {
    let method_id = env
        .get_method_id(class, name, sig)
        // we need this line to erase lifetime in order to save underlying raw pointer in static
        .map(|mid| mid.into_inner().into())
        .unwrap_or_else(|_| {
            panic!(
                "Method {} with signature {} of class {} not found",
                name, sig, class
            )
        });
    Some(method_id)
}

fn get_static_method_id(env: &JNIEnv, class: &str, name: &str, sig: &str) -> Option<JStaticMethodID<'static>> {
    let method_id = env
        .get_static_method_id(class, name, sig)
        // we need this line to erase lifetime in order to save underlying raw pointer in static
        .map(|mid| mid.into_inner().into())
        .unwrap_or_else(|_| {
            panic!(
                "Static method {} with signature {} of class {} not found",
                name, sig, class
            )
        });
    Some(method_id)
}

/// Returns cached class reference.
///
/// Always returns Some(class_ref), panics if class not found.
fn get_class(env: &JNIEnv, class: &str) -> Option<GlobalRef> {
    let class = env
        .find_class(class)
        .unwrap_or_else(|_| panic!("Class {} not found", class));
    Some(env.new_global_ref(class).unwrap())
}

fn check_cache_initialized() {
    if !INIT.state().done() {
        panic!("JNI cache is not initialized")
    }
}
