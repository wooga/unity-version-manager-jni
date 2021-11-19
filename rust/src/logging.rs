use flexi_logger::{default_format, DeferredNow, Logger, LogTarget};
use flexi_logger::writers::LogWriter;
use jni::JNIEnv;
use jni::signature::{JavaType, Primitive};
use log::{LevelFilter, Record};

use crate::jni_cache::{logging, system};

struct JniLogWriter {}

impl LogWriter for JniLogWriter {
    fn write(&self, _now: &mut DeferredNow, record: &Record) -> std::io::Result<()> {
        if let Ok(env) = system::get_jvm().get_env() {
            let method = match record.level() {
                log::Level::Trace => logging::get_fine_id(),
                log::Level::Debug => logging::get_fine_id(),
                log::Level::Info => logging::get_info_id(),
                log::Level::Warn => logging::get_warning_id(),
                log::Level::Error => logging::get_severe_id(),
            };

            let message = env.new_string(format!("{}", record.args())).unwrap();
            env.call_method_unchecked(logging::get_logger_obj(), method, JavaType::Primitive(Primitive::Void), &[message.into()]);
        } else {
            eprintln!("JVM LOG {}", record.args());
        }

        Ok(())
    }

    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }

    fn max_log_level(&self) -> LevelFilter {
        LevelFilter::max()
    }
}

pub fn setup_logger() {
    Logger::with_env_or_str("warn,uvm_core=debug,uvm_install2=debug,uvm_move_dir=debug,uvm_jni=debug")
        .log_target(LogTarget::Writer(Box::new(JniLogWriter {})))
        .format(default_format)
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));
}
