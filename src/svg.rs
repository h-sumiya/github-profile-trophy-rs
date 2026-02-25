use std::fmt::Write;

use crate::{
    constants::DEFAULT_PANEL_SIZE,
    models::UserInfo,
    themes::Theme,
    trophy::{Rank, Trophy, TrophyList},
};

const LEAF_ICON_TEMPLATE: &str = include_str!("leaf_icon.template.svg");

#[derive(Debug, Clone)]
pub struct Card {
    titles: Vec<String>,
    ranks: Vec<String>,
    max_column: i32,
    max_row: i32,
    panel_size: i32,
    margin_width: i32,
    margin_height: i32,
    no_background: bool,
    no_frame: bool,
}

impl Card {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        titles: Vec<String>,
        ranks: Vec<String>,
        max_column: i32,
        max_row: i32,
        panel_size: i32,
        margin_width: i32,
        margin_height: i32,
        no_background: bool,
        no_frame: bool,
    ) -> Self {
        Self {
            titles,
            ranks,
            max_column,
            max_row,
            panel_size,
            margin_width,
            margin_height,
            no_background,
            no_frame,
        }
    }

    pub fn render(&self, user_info: &UserInfo, theme: &Theme) -> String {
        let mut trophy_list = TrophyList::new(user_info);

        trophy_list.filter_by_hidden();

        if !self.titles.is_empty() {
            let include_titles = self
                .titles
                .iter()
                .filter(|title| !title.starts_with('-'))
                .cloned()
                .collect::<Vec<_>>();

            if !include_titles.is_empty() {
                trophy_list.filter_by_titles(&include_titles);
            }

            trophy_list.filter_by_exclusion_titles(&self.titles);
        }

        if !self.ranks.is_empty() {
            trophy_list.filter_by_ranks(&self.ranks);
        }

        trophy_list.sort_by_rank();

        let mut max_column = if self.max_column == -1 {
            (trophy_list.len() as i32).max(1)
        } else if self.max_column <= 0 {
            1
        } else {
            self.max_column
        };

        if max_column <= 0 {
            max_column = 1;
        }

        let width = self.panel_size * max_column + self.margin_width * (max_column - 1);

        let row = get_row(trophy_list.len(), max_column, self.max_row);
        let height = get_height(self.panel_size, self.margin_height, row);

        let body = self.render_trophies(trophy_list.items(), theme, max_column);

        format!(
            "\n    <svg\n      width=\"{width}\"\n      height=\"{height}\"\n      viewBox=\"0 0 {width} {height}\"\n      fill=\"none\"\n      xmlns=\"http://www.w3.org/2000/svg\"\n    >\n      {body}\n    </svg>"
        )
    }

    fn render_trophies(&self, trophies: &[Trophy], theme: &Theme, max_column: i32) -> String {
        let mut output = String::with_capacity(trophies.len() * 2_500);

        for (index, trophy) in trophies.iter().enumerate() {
            let current_column = (index as i32) % max_column;
            let current_row = (index as i32) / max_column;
            let x = self.panel_size * current_column + self.margin_width * current_column;
            let y = self.panel_size * current_row + self.margin_height * current_row;

            output.push_str(&render_trophy(
                trophy,
                theme,
                x,
                y,
                self.panel_size,
                self.no_background,
                self.no_frame,
            ));
        }

        output
    }
}

fn get_row(trophy_count: usize, max_column: i32, max_row: i32) -> i32 {
    if trophy_count == 0 {
        return 1;
    }

    let mut row = (((trophy_count as i32) - 1) / max_column) + 1;
    if row > max_row {
        row = max_row;
    }
    row.max(1)
}

fn get_height(panel_size: i32, margin_height: i32, row: i32) -> i32 {
    panel_size * row + margin_height * (row - 1)
}

fn render_trophy(
    trophy: &Trophy,
    theme: &Theme,
    x: i32,
    y: i32,
    panel_size: i32,
    no_background: bool,
    no_frame: bool,
) -> String {
    let next_rank_bar = get_next_rank_bar(
        trophy.title,
        trophy.calculate_next_rank_percentage(),
        theme.next_rank_bar,
    );

    let trophy_icon = get_trophy_icon(theme, trophy.rank);

    let frame_opacity = if no_frame { "0" } else { "1" };
    let background_opacity = if no_background { "0" } else { "1" };

    format!(
        "\n        <svg\n          x=\"{x}\"\n          y=\"{y}\"\n          width=\"{panel_size}\"\n          height=\"{panel_size}\"\n          viewBox=\"0 0 {panel_size} {panel_size}\"\n          fill=\"none\"\n          xmlns=\"http://www.w3.org/2000/svg\"\n        >\n          <rect\n            x=\"0.5\"\n            y=\"0.5\"\n            rx=\"4.5\"\n            width=\"{}\"\n            height=\"{}\"\n            stroke=\"#e1e4e8\"\n            fill=\"{}\"\n            stroke-opacity=\"{frame_opacity}\"\n            fill-opacity=\"{background_opacity}\"\n          />\n          {trophy_icon}\n          <text x=\"50%\" y=\"18\" text-anchor=\"middle\" font-family=\"Segoe UI,Helvetica,Arial,sans-serif,Apple Color Emoji,Segoe UI Emoji\" font-weight=\"bold\" font-size=\"13\" fill=\"{}\">{}</text>\n          <text x=\"50%\" y=\"85\" text-anchor=\"middle\" font-family=\"Segoe UI,Helvetica,Arial,sans-serif,Apple Color Emoji,Segoe UI Emoji\" font-weight=\"bold\" font-size=\"10.5\" fill=\"{}\">{}</text>\n          <text x=\"50%\" y=\"97\" text-anchor=\"middle\" font-family=\"Segoe UI,Helvetica,Arial,sans-serif,Apple Color Emoji,Segoe UI Emoji\" font-weight=\"bold\" font-size=\"10\" fill=\"{}\">{}</text>\n          {next_rank_bar}\n        </svg>\n        ",
        panel_size - 1,
        panel_size - 1,
        theme.background,
        theme.title,
        trophy.title,
        theme.text,
        trophy.top_message,
        theme.text,
        trophy.bottom_message,
    )
}

fn get_next_rank_bar(title: &str, percentage: f64, color: &str) -> String {
    let max_width = 80.0;
    let progress_width = max_width * percentage;

    format!(
        "\n    <style>\n    @keyframes {title}RankAnimation {{\n      from {{\n        width: 0px;\n      }}\n      to {{\n        width: {progress_width}px;\n      }}\n    }}\n    #{title}-rank-progress{{\n      animation: {title}RankAnimation 1s forwards ease-in-out;\n    }}\n    </style>\n    <rect\n      x=\"15\"\n      y=\"101\"\n      rx=\"1\"\n      width=\"{max_width}\"\n      height=\"3.2\"\n      opacity=\"0.3\"\n      fill=\"{color}\"\n    />\n    <rect\n      id=\"{title}-rank-progress\"\n      x=\"15\"\n      y=\"101\"\n      rx=\"1\"\n      height=\"3.2\"\n      fill=\"{color}\"\n    />\n  "
    )
}

fn get_trophy_icon(theme: &Theme, rank: Rank) -> String {
    let mut color = theme.default_rank_base;
    let mut rank_color = theme.default_rank_text;
    let mut background_icon = String::new();
    let mut gradation_color = format!(
        "\n      <stop offset=\"0%\" stop-color=\"{}\"/>\n      <stop offset=\"50%\" stop-color=\"{}\"/>\n      <stop offset=\"100%\" stop-color=\"{}\"/>\n  ",
        theme.default_rank_base, theme.default_rank_base, theme.default_rank_shadow
    );

    if rank == Rank::Secret {
        rank_color = theme.secret_rank_text;
        gradation_color = format!(
            "\n    <stop offset=\"0%\" stop-color=\"{}\"/>\n    <stop offset=\"50%\" stop-color=\"{}\"/>\n    <stop offset=\"100%\" stop-color=\"{}\"/>\n    ",
            theme.secret_rank_1, theme.secret_rank_2, theme.secret_rank_3
        );
    } else if rank.first_letter() == "S" {
        color = theme.s_rank_base;
        rank_color = theme.s_rank_text;
        background_icon = leaf_icon(theme.laurel);
        gradation_color = format!(
            "\n    <stop offset=\"0%\" stop-color=\"{color}\"/>\n    <stop offset=\"70%\" stop-color=\"{color}\"/>\n    <stop offset=\"100%\" stop-color=\"{}\"/>\n    ",
            theme.s_rank_shadow
        );
    } else if rank.first_letter() == "A" {
        color = theme.a_rank_base;
        rank_color = theme.a_rank_text;
        background_icon = leaf_icon(theme.laurel);
        gradation_color = format!(
            "\n    <stop offset=\"0%\" stop-color=\"{color}\"/>\n    <stop offset=\"70%\" stop-color=\"{color}\"/>\n    <stop offset=\"100%\" stop-color=\"{}\"/>\n    ",
            theme.a_rank_shadow
        );
    } else if rank == Rank::B {
        color = theme.b_rank_base;
        rank_color = theme.b_rank_text;
        gradation_color = format!(
            "\n    <stop offset=\"0%\" stop-color=\"{color}\"/>\n    <stop offset=\"70%\" stop-color=\"{color}\"/>\n    <stop offset=\"100%\" stop-color=\"{}\"/>\n    ",
            theme.b_rank_shadow
        );
    }

    let icon = format!(
        "\n    <path d=\"M7 10h2v4H7v-4z\"/>\n    <path d=\"M10 11c0 .552-.895 1-2 1s-2-.448-2-1 .895-1 2-1 2 .448 2 1z\"/>\n    <path fill-rule=\"evenodd\" d=\"M12.5 3a2 2 0 1 0 0 4 2 2 0 0 0 0-4zm-3 2a3 3 0 1 1 6 0 3 3 0 0 1-6 0zm-6-2a2 2 0 1 0 0 4 2 2 0 0 0 0-4zm-3 2a3 3 0 1 1 6 0 3 3 0 0 1-6 0z\"/>\n    <path d=\"M3 1h10c-.495 3.467-.5 10-5 10S3.495 4.467 3 1zm0 15a1 1 0 0 1 1-1h8a1 1 0 0 1 1 1H3zm2-1a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1H5z\"/>\n    <circle cx=\"8\" cy=\"6\" r=\"4\" fill=\"{}\" />\n    <text x=\"6\" y=\"8\" font-family=\"Courier, Monospace\" font-size=\"7\" fill=\"{rank_color}\">{}</text>\n  ",
        theme.icon_circle,
        rank.first_letter()
    );

    let option_rank_icon =
        get_small_trophy_icon(&icon, color, rank.as_str().len().saturating_sub(1));

    let mut output = String::with_capacity(1_000 + background_icon.len() + option_rank_icon.len());
    output.push_str(&background_icon);
    output.push_str(&option_rank_icon);

    let _ = write!(
        output,
        "\n  <defs>\n    <linearGradient id=\"{}\" gradientTransform=\"rotate(45)\">\n    {gradation_color}\n    </linearGradient>\n  </defs>\n  <svg x=\"28\" y=\"20\" width=\"100\" height=\"100\" viewBox=\"0 0 30 30\" fill=\"url(#{})\" xmlns=\"http://www.w3.org/2000/svg\">\n    {icon}\n  </svg>\n  ",
        rank.as_str(),
        rank.as_str(),
    );

    output
}

fn get_small_trophy_icon(icon: &str, color: &str, count: usize) -> String {
    let left_x_position = 7;
    let right_x_position = 68;

    let render_icon = |x: i32| {
        format!(
            "<svg x=\"{x}\" y=\"35\" width=\"65\" height=\"65\" viewBox=\"0 0 30 30\" fill=\"{color}\" xmlns=\"http://www.w3.org/2000/svg\">\n      {icon}\n    </svg>"
        )
    };

    match count {
        1 => render_icon(right_x_position),
        2 => format!(
            "{}{}",
            render_icon(left_x_position),
            render_icon(right_x_position)
        ),
        _ => String::new(),
    }
}

fn leaf_icon(laurel: &str) -> String {
    LEAF_ICON_TEMPLATE.replace("__LAUREL__", laurel)
}

#[allow(dead_code)]
pub fn render_cli_svg(user_info: &UserInfo, theme: &Theme) -> String {
    Card::new(
        Vec::new(),
        Vec::new(),
        -1,
        10,
        DEFAULT_PANEL_SIZE + 5,
        10,
        10,
        false,
        false,
    )
    .render(user_info, theme)
}
