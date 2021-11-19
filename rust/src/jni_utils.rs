use super::*;
use uvm_install2::unity;

/// Converts a `java.io.File` `JObject` into a `PathBuf`
pub fn get_path(env: &JNIEnv, path: JObject) -> error::UvmJniResult<PathBuf> {
    use jni_cache::io::file;
    env.call_method_unchecked(path, file::get_absolute_path_id(), file::get_absolute_path_type(), &[])
        .and_then(JValue::l)
        .and_then(|object| env.get_string(object.into()))
        .map(|p| Path::new(&String::from(p)).to_path_buf())
        .map_err(|e| e.into())
}

/// Constructs a `java.io.File` `JObject` from given path.
pub fn get_file<'a, 'b>(env: &'a JNIEnv<'b>, path: &'b Path) -> error::UvmJniResult<JObject<'b>> {
    use jni_cache::io::file;
    let class = file::get_file_class();
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

pub fn print_error_and_return_null(err: UvmJniError) -> jobject {
    error!("error: {}", err);

    for err in err.iter().skip(1) {
        error!("caused by: {}", err);
    }

    if let Some(backtrace) = err.backtrace() {
        error!("backtrace: {:?}", backtrace);
    }
    eprintln!("{}", err.description());

    JObject::null().into_inner()
}
