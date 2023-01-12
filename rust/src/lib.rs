use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Once;

use flexi_logger::{default_format, Logger};
use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::{jboolean, jint, jobject, jobjectArray, jsize, jstring};
use log::*;
use uvm_install2::unity::{Component};
use uvm_install2::unity::{Manifest, Modules, Version};

use error::*;

static START_LOGGER: Once = Once::new();

fn start_logger() {
    START_LOGGER.call_once(|| {
        Logger::with_env_or_str("warning")
            .format(default_format)
            .start()
            .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));
    });
}

mod error {
    use std;
    use jni;
    use thiserror::Error;
    use uvm_install2;

    #[derive(Error, Debug)]
    pub enum UvmJniError {
        #[error("installation error")]
        InstallError {
            #[from]
            source: uvm_install2::error::Error,
        },

        #[error("invalid unity version")]
        UnityVersionError {
            #[from]
            source: uvm_install2::unity::VersionError,
        },

        #[error("JNI error")]
        JNIError {
            #[from]
            source: jni::errors::Error,
        },

        #[error("io error")]
        Io {
            #[from]
            source: std::io::Error,
        },

        #[error("unknown error")]
        Other {
            #[from]
            source: uvm_install2::uvm_core_error::UvmError,
        },
    }

    pub type UvmJniResult<T> = std::result::Result<T, UvmJniError>;
}

mod jni_utils {
    use uvm_install2::unity;
    use std::error::Error;
    use super::*;

    /// Converts a `java.io.File` `JObject` into a `PathBuf`
    pub fn get_path(env: &JNIEnv, path: JObject) -> error::UvmJniResult<PathBuf> {
        env.call_method(path, "getAbsolutePath", "()Ljava/lang/String;", &[])
            .and_then(JValue::l)
            .and_then(|object| env.get_string(object.into()))
            .map(|p| Path::new(&String::from(p)).to_path_buf())
            .map_err(|e| e.into())
    }

    pub fn get_file<'a, 'b>(
        env: &'a JNIEnv<'b>,
        path: &'b Path,
    ) -> error::UvmJniResult<JObject<'b>> {
        let class = env.find_class("java/io/File")?;
        let path_string = env.new_string(path.to_string_lossy())?;
        let object = env.new_object(
            class,
            "(Ljava/lang/String;)V",
            &[JValue::Object(path_string.into())],
        )?;
        Ok(object)
    }

    pub fn get_installation<'a, 'b>(
        env: &'a JNIEnv<'b>,
        installation: &'b unity::Installation,
    ) -> error::UvmJniResult<JObject<'b>> {
        let installation_class = env.find_class("net/wooga/uvm/Installation")?;
        let install_path = jni_utils::get_file(&env, installation.path())?;
        let install_version = env.new_string(installation.version().to_string())?;
        let native_installation = env.new_object(
            installation_class,
            "(Ljava/io/File;Ljava/lang/String;)V",
            &[
                JValue::Object(install_path),
                JValue::Object(install_version.into()),
            ],
        )?;
        Ok(native_installation)
    }

    pub fn get_component<'a, 'b>(
        env: &'a JNIEnv<'b>,
        component: Component,
    ) -> error::UvmJniResult<JObject<'a>> {
        use Component::*;
        let component_class = env.find_class("net/wooga/uvm/Component")?;
        let component_method = match component {
            Android => "android",
            Ios => "ios",
            TvOs => "tvOs",
            WebGl => "webGl",
            Linux => "linux",
            Windows => "windows",
            WindowsMono => "windowsMono",
            Editor => "editor",
            Mono => "mono",
            VisualStudio => "visualStudio",
            MonoDevelop => "monoDevelop",
            StandardAssets => "standardAssets",
            Documentation => "documentation",
            #[cfg(windows)]
            VisualStudioProfessionalUnityWorkload => "visualStudioProfessionalUnityWorkload",
            #[cfg(windows)]
            VisualStudioEnterpriseUnityWorkload => "visualStudioProfessionalUnityWorkload",
            ExampleProject => "exampleProject",
            Example => "exampleProject",
            AndroidSdkNdkTools => "androidSdkNdkTools",
            AndroidSdkPlatforms => "androidSdkPlatforms",
            AndroidSdkPlatformTools => "androidSdkPlatformTools",
            AndroidSdkBuildTools => "androidSdkBuildTools",
            AndroidNdk => "androidNdk",
            AndroidOpenJdk => "androidOpenJdk",
            AppleTV => "appleTV",
            LinuxMono => "linuxMono",
            Mac => "mac",
            MacIL2CPP => "macIL2CPP",
            MacMono => "macMono",
            #[cfg(windows)]
            Metro => "metro",
            #[cfg(windows)]
            UwpIL2CPP => "uwpIL2CPP",
            #[cfg(windows)]
            UwpNet => "uwpNet",
            #[cfg(windows)]
            UniversalWindowsPlatform => "universalWindowsPlatform",
            Samsungtv => "samsungtv",
            SamsungTV => "samsungTV",
            Tizen => "tizen",
            Vuforia => "vuforia",
            VuforiaAR => "vuforiaAR",
            #[cfg(windows)]
            WindowsIL2CCP => "windowsIL2CCP",
            Facebook => "facebook",
            FacebookGames => "facebookGames",
            FacebookGameRoom => "facebookGameRoom",
            Lumin => "lumin",
            _ => "unknown",
        };
        let native_component = env.get_static_field(
            component_class,
            component_method,
            "Lnet/wooga/uvm/Component;",
        )?;
        let native_component = native_component.l()?;
        Ok(native_component)
    }

    pub fn get_components<I: IntoIterator<Item=Component>> (env: &JNIEnv, components: I) -> UvmJniResult<jobjectArray> {
        let component_class = env.find_class("net/wooga/uvm/Component")?;
        let components: Vec<Component> = components.into_iter().collect();

        let output =
            env.new_object_array(components.len() as jsize, component_class, JObject::null())?;
        for (i, component) in components.into_iter().enumerate() {
            let native_component = get_component(&env, component)?;
            env.set_object_array_element(output, i as jsize, native_component)?;
        }
        return Ok(output);
    }

    pub fn print_error_and_return_null(err: UvmJniError) -> jobject {
        eprintln!("error: {}", err);

        if let Some(cause) = err.source() {
            error!("caused by: {}", cause);
        }

        JObject::null().into_inner()
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_wooga_uvm_UnityVersionManager_uvmVersion(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    start_logger();
    env.new_string("0.6.0")
        .map(|s| s.into_inner())
        .map_err(|e| e.into())
        .unwrap_or_else(jni_utils::print_error_and_return_null)
}

fn list_installations(env: &JNIEnv) -> error::UvmJniResult<jobjectArray> {
    let installations = uvm_install2::list_all_installations()?;
    let installations: Vec<uvm_install2::unity::Installation> = installations.collect();
    let installation_class = env.find_class("net/wooga/uvm/Installation")?;

    let output = env.new_object_array(
        installations.len() as jsize,
        installation_class,
        JObject::null(),
    )?;
    for (i, installation) in installations.iter().enumerate() {
        let native_installation = jni_utils::get_installation(&env, &installation)?;
        env.set_object_array_element(output, i as jsize, native_installation)?;
    }

    Ok(output)
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_wooga_uvm_UnityVersionManager_listInstallations(
    env: JNIEnv,
    _class: JClass,
) -> jobjectArray {
    start_logger();
    list_installations(&env).unwrap_or_else(jni_utils::print_error_and_return_null)
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_wooga_uvm_UnityVersionManager_detectProjectVersion(
    env: JNIEnv,
    _class: JClass,
    path: JObject,
    with_revision_hash: jboolean,
) -> jstring {
    start_logger();
    jni_utils::get_path(&env, path)
        .and_then(|path| {
            uvm_install2::dectect_project_version(&path, Some(true)).map_err(|e| e.into())
        })
        .and_then(|version| {
            let version = if with_revision_hash == 0 {
                version.to_string()
            } else {
                if let Ok(hash) = version.version_hash() {
                    format!("{} ({})", version, hash)
                } else {
                    version.to_string()
                }
            };
            env.new_string(version).map_err(|e| e.into())
        })
        .map(|s| s.into_inner())
        .unwrap_or_else(jni_utils::print_error_and_return_null)
}

fn locate_installation(env: &JNIEnv, version: JString) -> error::UvmJniResult<jobject> {
    let version_string = env.get_string(version)?;
    let version_string: String = version_string.into();
    let version = Version::from_str(&version_string)?;
    let installation = uvm_install2::find_installation(&version)?;

    let native_installation = jni_utils::get_installation(&env, &installation)?;
    Ok(native_installation.into_inner())
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_wooga_uvm_UnityVersionManager_locateUnityInstallation(
    env: JNIEnv,
    _class: JClass,
    version: JString,
) -> jobject {
    start_logger();
    locate_installation(&env, version).unwrap_or_else(jni_utils::print_error_and_return_null)
}

#[derive(Debug, Eq, PartialEq, Hash)]
struct Variant(Component);

impl AsRef<Component> for Variant {
    fn as_ref(&self) -> &Component {
        &self.0
    }
}

impl From<jint> for Variant {
    fn from(component: jint) -> Self {
        use Component::*;
        match component {
            0 => Variant(Android),
            1 => Variant(Ios),
            3 => Variant(WebGl),
            4 => Variant(Linux),
            5 => Variant(Windows),
            6 => Variant(WindowsMono),
            7 => Variant(Editor),
            8 => Variant(Mono),
            9 => Variant(VisualStudio),
            10 => Variant(MonoDevelop),
            11 => Variant(StandardAssets),
            12 => Variant(Documentation),
            #[cfg(windows)]
            13 => Variant(VisualStudioEnterpriseUnityWorkload),
            #[cfg(windows)]
            14 => Variant(VisualStudioEnterpriseUnityWorkload),
            15 => Variant(ExampleProject),
            16 => Variant(Example),
            17 => Variant(AndroidSdkNdkTools),
            18 => Variant(AndroidSdkPlatforms),
            19 => Variant(AndroidSdkPlatformTools),
            20 => Variant(AndroidSdkBuildTools),
            21 => Variant(AndroidNdk),
            22 => Variant(AndroidOpenJdk),
            23 => Variant(AppleTV),
            24 => Variant(LinuxMono),
            25 => Variant(Mac),
            26 => Variant(MacIL2CPP),
            27 => Variant(MacMono),
            #[cfg(windows)]
            28 => Variant(Metro),
            #[cfg(windows)]
            29 => Variant(UwpIL2CPP),
            #[cfg(windows)]
            30 => Variant(UwpNet),
            #[cfg(windows)]
            31 => Variant(UniversalWindowsPlatform),
            32 => Variant(Samsungtv),
            33 => Variant(SamsungTV),
            34 => Variant(Tizen),
            35 => Variant(Vuforia),
            36 => Variant(VuforiaAR),
            #[cfg(windows)]
            37 => Variant(WindowsIL2CCP),
            38 => Variant(Facebook),
            39 => Variant(FacebookGames),
            40 => Variant(FacebookGameRoom),
            41 => Variant(Lumin),
            42 => Variant(LinuxIL2CPP),
            43 => Variant(LinuxServer),
            44 => Variant(MacServer),
            45 => Variant(WindowsServer),
            _ => Variant(Unknown),
        }
    }
}

fn install_unity_editor(
    env: &JNIEnv,
    version: JString,
    destination: Option<JObject>,
    components: Option<jobjectArray>,
) -> error::UvmJniResult<jobject> {
    let version = env.get_string(version)?;
    let version: String = version.into();
    let version = Version::from_str(&version)?;

    let destination = if let Some(destination) = destination {
        jni_utils::get_path(env, destination).ok()
    } else {
        None
    };

    let variants = if let Some(components) = components {
        let length = env.get_array_length(components)?;
        let mut variants: HashSet<Variant> = HashSet::with_capacity(length as usize);
        for i in 0..length {
            let item = env.get_object_array_element(components, i)?;
            let item_value = env.call_method(item, "value", "()I", &[])?;
            let item_value: jint = item_value.i()?;
            let variant: Variant = item_value.into();
            variants.insert(variant);
        }
        Some(variants)
    } else {
        None
    };

    let installation = uvm_install2::install(&version, variants, true, destination.as_ref())?;
    let native_installation = jni_utils::get_installation(&env, &installation)?;
    Ok(native_installation.into_inner())
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_wooga_uvm_UnityVersionManager_installUnityEditor__Ljava_lang_String_2Ljava_io_File_2(
    env: JNIEnv,
    _class: JClass,
    version: JString,
    destination: JObject,
) -> jobject {
    start_logger();
    install_unity_editor(&env, version, Some(destination), None)
        .unwrap_or_else(jni_utils::print_error_and_return_null)
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_wooga_uvm_UnityVersionManager_installUnityEditor__Ljava_lang_String_2(
    env: JNIEnv,
    _class: JClass,
    version: JString,
) -> jobject {
    start_logger();
    install_unity_editor(&env, version, None, None)
        .unwrap_or_else(jni_utils::print_error_and_return_null)
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_wooga_uvm_UnityVersionManager_installUnityEditor__Ljava_lang_String_2_3Lnet_wooga_uvm_Component_2(
    env: JNIEnv,
    _class: JClass,
    version: JString,
    components: jobjectArray,
) -> jobject {
    start_logger();
    install_unity_editor(&env, version, None, Some(components))
        .unwrap_or_else(jni_utils::print_error_and_return_null)
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_wooga_uvm_UnityVersionManager_installUnityEditor__Ljava_lang_String_2Ljava_io_File_2_3Lnet_wooga_uvm_Component_2(
    env: JNIEnv,
    _class: JClass,
    version: JString,
    destination: JObject,
    components: jobjectArray,
) -> jobject {
    start_logger();
    install_unity_editor(&env, version, Some(destination), Some(components))
        .unwrap_or_else(jni_utils::print_error_and_return_null)
}

fn get_installation_components(env: &JNIEnv, object: JObject) -> error::UvmJniResult<jobjectArray> {
    let location = env.call_method(object, "getLocation", "()Ljava/io/File;", &[])?;
    let location = location.l()?;
    let path = jni_utils::get_path(&env, location)?;

    let installation = uvm_install2::unity::Installation::new(path)?;
    let components = uvm_install2::unity::InstalledComponents::new(installation);

    jni_utils::get_components(env, components)
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_wooga_uvm_Installation_getComponents(
    env: JNIEnv,
    object: JObject,
) -> jobjectArray {
    start_logger();
    get_installation_components(&env, object).unwrap_or_else(jni_utils::print_error_and_return_null)
}

fn get_installation(env: &JNIEnv, path: JObject) -> error::UvmJniResult<jobject> {
    let path = jni_utils::get_path(&env, path)?;
    let installation = uvm_install2::unity::Installation::new(path)?;
    let native_installation = jni_utils::get_installation(&env, &installation)?;
    Ok(native_installation.into_inner())
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_wooga_uvm_Installation_atLocation(
    env: JNIEnv,
    _class: JClass,
    path: JObject,
) -> jobject {
    start_logger();
    get_installation(&env, path).unwrap_or_else(jni_utils::print_error_and_return_null)
}

fn get_installation_executable(env: &JNIEnv, object: JObject) -> error::UvmJniResult<jobjectArray> {
    let location = env.call_method(object, "getLocation", "()Ljava/io/File;", &[])?;
    let location = location.l()?;
    let path = jni_utils::get_path(&env, location)?;

    let installation = uvm_install2::unity::Installation::new(path)?;
    let exec_path = installation.exec_path();
    let exec_path = jni_utils::get_file(&env, &exec_path)?;
    Ok(exec_path.into_inner())
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_wooga_uvm_Installation_getExecutable(
    env: JNIEnv,
    object: JObject,
) -> jobject {
    start_logger();
    get_installation_executable(&env, object).unwrap_or_else(jni_utils::print_error_and_return_null)
}

fn list_unity_components(env: &JNIEnv, version: JString) -> error::UvmJniResult<jobjectArray> {
    let version_string = env.get_string(version)?;
    let version_string: String = version_string.into();
    let version = Version::from_str(&version_string)?;
    let manifest = Manifest::load(&version)
        .map_err(|_e| io::Error::new(io::ErrorKind::NotFound, "failed to load manifest"))?;
    let modules: Modules = manifest.into_modules();

    let components = modules.iter().map(|m| m.id);
    let java_components: jobjectArray = jni_utils::get_components(env, components)?;
    Ok(java_components)
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_wooga_uvm_UnityVersionManager_listAvailableComponents(env: JNIEnv, _class: JClass, version: JString) -> jobjectArray {
    start_logger();
    let res = list_unity_components(&env, version);

    res.unwrap_or_else(jni_utils::print_error_and_return_null)
}
