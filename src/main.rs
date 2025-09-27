use std::collections::HashMap;

use openaction::*;

use sysinfo::System;

struct CPUAction;
#[async_trait]
impl Action for CPUAction {
	const UUID: ActionUuid = "me.amankhanna.oasystem.cpu";
	type Settings = HashMap<String, String>;
}

struct RAMAction;
#[async_trait]
impl Action for RAMAction {
	const UUID: ActionUuid = "me.amankhanna.oasystem.ram";
	type Settings = HashMap<String, String>;
}

struct UptimeAction;
#[async_trait]
impl Action for UptimeAction {
	const UUID: ActionUuid = "me.amankhanna.oasystem.uptime";
	type Settings = HashMap<String, String>;
}

struct OSAction;
#[async_trait]
impl Action for OSAction {
	const UUID: ActionUuid = "me.amankhanna.oasystem.os";
	type Settings = HashMap<String, String>;

	async fn will_appear(
		&self,
		instance: &Instance,
		_settings: &Self::Settings,
	) -> OpenActionResult<()> {
		instance
			.set_title(
				Some(
					System::long_os_version()
						.unwrap_or_else(|| "Unknown".to_owned())
						.replace(" ", "\n"),
				),
				None,
			)
			.await
	}
}

#[tokio::main]
async fn main() -> OpenActionResult<()> {
	{
		use simplelog::*;
		if let Err(error) = TermLogger::init(
			LevelFilter::Debug,
			Config::default(),
			TerminalMode::Stdout,
			ColorChoice::Never,
		) {
			eprintln!("Logger initialization failed: {}", error);
		}
	}

	tokio::spawn(async {
		let mut system = System::new_all();
		loop {
			system.refresh_cpu_usage();
			tokio::time::sleep(std::time::Duration::from_secs(1)).await;
			system.refresh_cpu_usage();

			let cpu_usage = format!("{:.0}%", system.global_cpu_usage());
			for instance in visible_instances(CPUAction::UUID).await {
				let _ = instance.set_title(Some(cpu_usage.clone()), None).await;
			}

			system.refresh_memory();
			let ram_usage = format!("{:.1}GB", (system.used_memory() as f32) / 1073741824.0);
			for instance in visible_instances(RAMAction::UUID).await {
				let _ = instance.set_title(Some(ram_usage.clone()), None).await;
			}

			{
				let total_secs = System::uptime();
				let days = total_secs / 86_400;
				let hours = (total_secs % 86_400) / 3600;
				let minutes = (total_secs % 3600) / 60;
				let seconds = total_secs % 60;

				for instance in visible_instances(UptimeAction::UUID).await {
					let _ = instance
						.set_title(
							Some(format!("{days}d {hours:02}h\n{minutes:02}m {seconds:02}s")),
							None,
						)
						.await;
				}
			}
		}
	});

	register_action(CPUAction).await;
	register_action(RAMAction).await;
	register_action(UptimeAction).await;
	register_action(OSAction).await;

	run(std::env::args().collect()).await
}
