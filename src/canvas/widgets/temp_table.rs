use lazy_static::lazy_static;
use std::cmp::max;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    terminal::Frame,
    widgets::{Block, Borders, Row, Table, Widget},
};

use crate::{
    app,
    canvas::{
        drawing_utils::{get_start_position, get_variable_intrinsic_widths},
        Painter,
    },
    constants::*,
};

const TEMP_HEADERS: [&str; 2] = ["Sensor", "Temp"];

lazy_static! {
    static ref TEMP_HEADERS_LENS: Vec<usize> = TEMP_HEADERS
        .iter()
        .map(|entry| max(FORCE_MIN_THRESHOLD, entry.len()))
        .collect::<Vec<_>>();
}
pub trait TempTableWidget {
    fn draw_temp_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    );
}

impl TempTableWidget for Painter {
    fn draw_temp_table<B: Backend>(
        &self, f: &mut Frame<'_, B>, app_state: &mut app::App, draw_loc: Rect, draw_border: bool,
        widget_id: u64,
    ) {
        if let Some(temp_widget_state) = app_state.temp_state.widget_states.get_mut(&widget_id) {
            let temp_sensor_data: &mut [Vec<String>] = &mut app_state.canvas_data.temp_sensor_data;

            let num_rows = max(0, i64::from(draw_loc.height) - 5) as u64;
            let start_position = get_start_position(
                num_rows,
                &temp_widget_state.scroll_state.scroll_direction,
                &mut temp_widget_state.scroll_state.previous_scroll_position,
                temp_widget_state.scroll_state.current_scroll_position,
                app_state.is_resized,
            );

            let sliced_vec = &temp_sensor_data[start_position as usize..];
            let mut temp_row_counter: i64 = 0;
            let current_widget_id = app_state.current_widget.widget_id;

            let temperature_rows = sliced_vec.iter().map(|temp_row| {
                Row::StyledData(
                    temp_row.iter(),
                    if current_widget_id == widget_id {
                        if temp_row_counter as u64
                            == temp_widget_state.scroll_state.current_scroll_position
                                - start_position
                        {
                            temp_row_counter = -1;
                            self.colours.currently_selected_text_style
                        } else {
                            if temp_row_counter >= 0 {
                                temp_row_counter += 1;
                            }
                            self.colours.text_style
                        }
                    } else {
                        self.colours.text_style
                    },
                )
            });

            // Calculate widths
            let width = f64::from(draw_loc.width);
            let width_ratios = [0.5, 0.5];
            let variable_intrinsic_results =
                get_variable_intrinsic_widths(width as u16, &width_ratios, &TEMP_HEADERS_LENS);
            let intrinsic_widths = &(variable_intrinsic_results.0)[0..variable_intrinsic_results.1];

            let title = if app_state.is_expanded {
                const TITLE_BASE: &str = " Temperatures ── Esc to go back ";
                let repeat_num = max(
                    0,
                    draw_loc.width as i32 - TITLE_BASE.chars().count() as i32 - 2,
                );
                let result_title = format!(
                    " Temperatures ─{}─ Esc to go back ",
                    "─".repeat(repeat_num as usize)
                );

                result_title
            } else if app_state.app_config_fields.use_basic_mode {
                String::new()
            } else {
                " Temperatures ".to_string()
            };

            let temp_block = if draw_border {
                Block::default()
                    .title(&title)
                    .title_style(if app_state.is_expanded {
                        if app_state.current_widget.widget_id == widget_id {
                            self.colours.highlighted_border_style
                        } else {
                            self.colours.border_style
                        }
                    } else {
                        self.colours.widget_title_style
                    })
                    .borders(Borders::ALL)
                    .border_style(if app_state.current_widget.widget_id == widget_id {
                        self.colours.highlighted_border_style
                    } else {
                        self.colours.border_style
                    })
            } else if app_state.current_widget.widget_id == widget_id {
                Block::default()
                    .borders(*SIDE_BORDERS)
                    .border_style(self.colours.highlighted_border_style)
            } else {
                Block::default().borders(Borders::NONE)
            };

            let margined_draw_loc = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .horizontal_margin(
                    if app_state.current_widget.widget_id == widget_id || draw_border {
                        0
                    } else {
                        1
                    },
                )
                .direction(Direction::Horizontal)
                .split(draw_loc);

            // Draw
            Table::new(TEMP_HEADERS.iter(), temperature_rows)
                .block(temp_block)
                .header_style(self.colours.table_header_style)
                .widths(
                    &(intrinsic_widths
                        .iter()
                        .map(|calculated_width| Constraint::Length(*calculated_width as u16))
                        .collect::<Vec<_>>()),
                )
                .render(f, margined_draw_loc[0]);
        }
    }
}
