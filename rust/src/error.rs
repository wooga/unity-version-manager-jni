use error_chain::*;
use jni;
use std;
use uvm_install2;

error_chain! {
    types {
        UvmJniError, UvmJniErrorKind, ResultExt, UvmJniResult;
    }

    links {
        Another(jni::errors::Error, jni::errors::ErrorKind);
        UvmCore(uvm_install2::uvm_core_error::UvmError, uvm_install2::uvm_core_error::UvmErrorKind);
        UvmInstall(uvm_install2::error::Error, uvm_install2::error::ErrorKind);
        UvmVersion(uvm_install2::unity::UvmVersionError, uvm_install2::unity::UvmVersionErrorKind);
    }

    foreign_links {
        Io(std::io::Error);
    }
}
