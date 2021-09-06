use chrono::NaiveDate;
use nu_ansi_term::Color;
use std::collections::BTreeMap;

use crate::data::activity;
use crate::conf;
use crate::view::format_util;
use crate::view::table;

// displays a table with activities
pub fn list_activities(activities: &[&activity::Activity], with_start_dates: bool) {
    if activities.is_empty() {
        println!("No activity to display");
        return;
    }

    let mut activity_table = table::Table::new(vec![
        "Started".to_string(),
        "Stopped".to_string(),
        "Description".to_string(),
        "Project".to_string(),
        "Duration".to_string(),
    ]);

    activities
        .iter()
        .map(|t| get_activity_table_row(&t, with_start_dates))
        .for_each(|row| activity_table.add_row(row));

    println!("\n{}", activity_table);
}

// list activities grouped by the dates of their start time
pub fn list_activities_grouped_by_date(activities: &[&activity::Activity]) {
    if activities.is_empty() {
        println!("No activity to display");
        return;
    }

    let mut activity_table = table::Table::new(vec![
        "Started".to_string(),
        "Stopped".to_string(),
        "Description".to_string(),
        "Project".to_string(),
        "Duration".to_string(),
    ]);

    group_activities_by_date(activities)
        .iter()
        .map(|(date, activity_list)| {
            create_activites_group(&format!("{}", date), activity_list.as_slice())
        })
        .for_each(|g| activity_table.add_group(g));

    println!("\n{}", activity_table);
}

fn create_activites_group(title: &str, activities: &[&activity::Activity]) -> table::Group {
    let rows = activities
        .iter()
        .map(|a| get_activity_table_row(&a, false))
        .collect();
    table::Group::new(Some(title.to_string()), rows)
}

// displays a table with running activities (no end time)
pub fn list_running_activities(running_activities: &[&activity::Activity]) {
    if running_activities.is_empty() {
        println!("No Activity is currently running");
    } else {
        let mut activity_table = table::Table::new(vec![
            "Started At".to_string(),
            "Description".to_string(),
            "Project".to_string(),
            "Duration".to_string(),
        ]);

        running_activities
            .iter()
            .map(|activity| {
                table::Row::new(vec![
                    activity.start.format(conf::FORMAT_DATETIME).to_string(),
                    activity.description.clone(),
                    activity.project.clone(),
                    format_util::format_duration(&activity.get_duration()),
                ])
            })
            .for_each(|row| activity_table.add_row(row));

        println!("\n{}", activity_table);
    }
}

// displays a single activity
pub fn display_single_activity(activity: &activity::Activity) {
    println!("Begin: {}", activity.start.format(conf::FORMAT_DATETIME));

    if let Some(end) = activity.end {
        println!("End: {}", end.format(conf::FORMAT_DATETIME));
        println!(
            "Duration: {}",
            format_util::format_duration(&activity.get_duration())
        );
    }

    println!("Project: {}", activity.project);
    println!("Description: {}", activity.description);
}

// create a row for a activity
//
// the date of the end is shown when it is not the same date as the start
fn get_activity_table_row(activity: &activity::Activity, with_start_dates: bool) -> table::Row {
    let more_then_one_day = activity.end.map_or(false, |end| activity.start.date() != end.date());

    let display_end = activity.end.map_or_else(
        || "-".to_string(),
        |end| {
            if more_then_one_day {
                end.format(conf::FORMAT_DATETIME).to_string()
            } else {
                end.format(conf::FORMAT_TIME).to_string()
            }
        },
    );

    let start_format = if with_start_dates {
        conf::FORMAT_DATETIME
    } else {
        conf::FORMAT_TIME
    };

    let mut new_row = table::Row::new(vec![
        activity.start.format(start_format).to_string(),
        display_end,
        activity.description.clone(),
        activity.project.clone(),
        format_util::format_duration(&activity.get_duration()),
    ]);

    if !activity.is_stopped() {
        new_row.set_color(Color::Green.normal());
    } else if more_then_one_day {
        new_row.set_color(Color::Yellow.normal());
    }

    new_row
}

// groups activities in vectors of activities that started at the same day
fn group_activities_by_date<'a>(
    activities: &[&'a activity::Activity],
) -> BTreeMap<NaiveDate, Vec<&'a activity::Activity>> {
    let mut activities_by_date = BTreeMap::new();

    for &activity in activities.iter() {
        activities_by_date
            .entry(activity.start.date())
            .or_insert(Vec::new())
            .push(activity);
    }

    for activity_list in activities_by_date.values_mut() {
        activity_list.sort_by_key(|activity| activity.start);
    }

    activities_by_date
}
