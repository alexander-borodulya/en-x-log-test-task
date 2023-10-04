use nxlog_task::task_1;
use nxlog_task::task_2;

fn main() {
    env_logger::init();
    task_1::run();
    task_2::run();
}
