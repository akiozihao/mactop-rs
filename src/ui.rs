
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    symbols,
    widgets::{Axis, Block, Chart, Dataset, Gauge, List},
    Frame,
};

use crate::app::App;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(frame.size());
    let sub_0_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(layout[0]);
    let sub_1_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(layout[1]);
    let sub_2_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(layout[2]);
    // let sub_3_layout = Layout::default()
    //     .direction(Direction::Horizontal)
    //     .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
    //     .split(layout[3]);
    // frame.render_widget(
    //     Paragraph::new(format!(
    //         "{}.{}.{}.{}.{}",
    //         app.mem.total, app.mem.used, app.mem.available, app.mem.swap_total, app.mem.swap_used
    //     ))
    //     .block(
    //         Block::bordered()
    //             .title("MemoryMetrics")
    //             .title_alignment(Alignment::Center)
    //             .border_type(BorderType::Rounded),
    //     )
    //     .style(Style::default().fg(Color::Cyan).bg(Color::Black))
    //     .centered(),
    //     frame.size(),
    // );
    frame.render_widget(
        Gauge::default()
            .block(Block::bordered().title(format!(
                "E-CPU Usage: {}% @{} MHz",
                app.metrics.cpu_metrics.e_cluster_active,
                app.metrics.cpu_metrics.e_cluster_freq_mhz
            )))
            .gauge_style(Style::default().on_black().green())
            .percent(app.metrics.cpu_metrics.e_cluster_active as u16),
        sub_0_layout[0],
    );
    frame.render_widget(
        Gauge::default()
            .block(Block::bordered().title(format!(
                "GPU Usage: {}% @ {} MHz",
                app.metrics.gpu_metrics.active, app.metrics.gpu_metrics.freq_mhz
            )))
            .gauge_style(Style::default().on_black().magenta())
            .percent(app.metrics.gpu_metrics.active as u16),
        sub_0_layout[1],
    );
    frame.render_widget(
        Gauge::default()
            .block(Block::bordered().title(format!(
                "P-CPU Usage: {}% @ {} MHz",
                app.metrics.cpu_metrics.p_cluster_active,
                app.metrics.cpu_metrics.p_cluster_freq_mhz
            )))
            .gauge_style(Style::default().on_black().magenta())
            .percent(app.metrics.cpu_metrics.p_cluster_active as u16),
        sub_1_layout[0],
    );
    frame.render_widget(
        Gauge::default()
            .block(Block::bordered().title(format!(
                "ANE Usage: {:.2}% @ {} W",
                app.metrics.cpu_metrics.ane_w * 100.0 / 8.0,
                app.metrics.cpu_metrics.ane_w
            )))
            .gauge_style(Style::default().on_black().magenta())
            .percent((app.metrics.cpu_metrics.ane_w * 100.0 / 8.0) as u16),
        sub_1_layout[1],
    );

    // apple silicon list
    let binding = "Unknown Model".to_string();
    let name = app.metrics.soc_info.get("name").unwrap_or(&binding);
    let binding = "0".to_string();
    let e_cores = app.metrics.soc_info.get("e_core_count").unwrap_or(&binding);
    let binding = "0".to_string();
    let p_cores = app.metrics.soc_info.get("p_core_count").unwrap_or(&binding);
    let total_cores =
        (e_cores.parse::<i32>().unwrap() + p_cores.parse::<i32>().unwrap()).to_string();
    let gpu_cores = app.metrics.soc_info.get("gpu_core_count").unwrap();
    let apple_silicon_item = [
        name.to_owned(),
        format!("Total cores: {total_cores}"),
        format!("E-Cores: {e_cores}"),
        format!("P-Cores: {p_cores}"),
        format!("GPU Cores: {gpu_cores}"),
    ];
    let apple_silicon_list =
        List::new(apple_silicon_item).block(Block::bordered().title("Apple Silicon"));
    frame.render_widget(apple_silicon_list, sub_2_layout[0]);

    // net work list
    let network_item = [
        format!(
            "Out: {:.1} packets/s, {:.1} bytes/s",
            app.metrics.net_disk_metrics.out_packets_per_sec,
            app.metrics.net_disk_metrics.out_bytes_per_sec
        ),
        format!(
            "In: {:.1} packets/s, {:.1} bytes/s",
            app.metrics.net_disk_metrics.in_packets_per_sec,
            app.metrics.net_disk_metrics.in_bytes_per_sec
        ),
        format!(
            "Read: {:.1} ops/s, {:.1} KBytes/s",
            app.metrics.net_disk_metrics.read_ops_per_sec,
            app.metrics.net_disk_metrics.read_k_bytes_per_sec
        ),
        format!(
            "Write: {:.1} ops/s, {:.1} KBytes/s",
            app.metrics.net_disk_metrics.write_ops_per_sec,
            app.metrics.net_disk_metrics.write_k_bytes_per_sec
        ),
    ];

    let network_list =
        List::new(network_item).block(Block::bordered().title("NetWork & Disk Info"));
    frame.render_widget(network_list, sub_2_layout[1]);

    // cpu-gpu list
    let cpu_w = app.metrics.cpu_metrics.cpu_w;
    let gpu_w = app.metrics.cpu_metrics.gpu_w;
    let ane_w = app.metrics.cpu_metrics.ane_w;
    let package_w = app.metrics.cpu_metrics.package_w;
    let cpu_gpu_item = [
        format!("CPU Power: {cpu_w} W"),
        format!("GPU Power: {gpu_w} W"),
        format!("ANE Power: {ane_w} W"),
        format!("Total Power: {package_w} W"),
    ];

    let network_list = List::new(cpu_gpu_item)
        .block(Block::bordered().title(format!("{} W CPU - {} W GPU", cpu_w, gpu_w)));
    frame.render_widget(network_list, sub_2_layout[2]);

    // total power
    let power_data: Vec<(f64, f64)> = app
        .cpu_w
        .iter()
        .enumerate()
        .map(|(index, &value)| (index as f64, value))
        .collect();
    let power_dataset = vec![Dataset::default()
        .marker(symbols::Marker::Dot)
        .style(Style::default().on_black().cyan())
        .data(&power_data[..])];
    let y_max = app.cpu_w.iter().copied().reduce(f64::max).unwrap_or(0.01);
    let power_chat = Chart::new(power_dataset)
        .block(Block::bordered().title(format!(
            "{:.2} W Total Power",
            app.cpu_w.last().unwrap_or(&0.0)
        )))
        .x_axis(Axis::default().bounds([0.0, 24.0]).labels(vec![
            "0".into(),
            // // "3".into(),
            // "5".into(),
            // // "9".into(),
            // "10".into(),
            // "15".into(),
            // "20".into(),
            "24".into(),
        ]))
        .y_axis(
            Axis::default()
                .bounds([0.0, y_max])
                .labels(vec!["0".into(), format!("{:.2}", y_max).into()]),
        );
    frame.render_widget(power_chat, sub_2_layout[3]);

    // memory usage
    frame.render_widget(
        Gauge::default()
            .block(Block::bordered().title(format!(
                "Memory Usage: {:.2} GB / {:.2} GB (Swap: {:.2}/{:.2} GB)",
                app.metrics.mem.used as f64 / 1024.0 / 1024.0 / 1024.0,
                app.metrics.mem.total as f64 / 1024.0 / 1024.0 / 1024.0,
                app.metrics.mem.swap_used as f64 / 1024.0 / 1024.0 / 1024.0,
                app.metrics.mem.swap_total as f64 / 1024.0 / 1024.0 / 1024.0,
            )))
            .gauge_style(Style::default().on_black().green())
            .percent(((app.metrics.mem.used as f64 / app.metrics.mem.total as f64) * 100.0) as u16),
        layout[3],
    )
}
