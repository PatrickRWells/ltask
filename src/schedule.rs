use chrono::{NaiveTime, Timelike, Weekday};
use std::collections::HashMap;
enum BlockType {
    Free,
    Busy,
}

struct TimeBlock {
    start: NaiveTime,
    end: NaiveTime,
    block_type: BlockType,
}

struct DaySchedule {
    blocks: Vec<TimeBlock>,
    interval_size: usize,
}

struct WeekSchedule {
    days: HashMap<Weekday, DaySchedule>,
}

fn index_to_time_range(index: usize, interval_size: usize) -> (NaiveTime, NaiveTime) {
    // We divide the day into 15 minute intervals, so there are a total of 96 intervals in a day.
    if index >= 1440 / interval_size {
        panic!("Time index out of range");
    } else if interval_size > 60 {
        panic!("Interval size cannot be greater than 60 minutes");
    }
    let start_minute = index * interval_size;
    let start = NaiveTime::from_hms_opt(
        (start_minute / 60).try_into().unwrap(),
        (start_minute % 60).try_into().unwrap(),
        0,
    )
    .unwrap();

    let end_minute = (index + 1) * interval_size;
    let end = if end_minute >= 1440 {
        NaiveTime::from_hms_opt(11, 59, 59).unwrap()
    } else {
        NaiveTime::from_hms_opt(
            (end_minute / 60).try_into().unwrap(),
            (end_minute % 60).try_into().unwrap(),
            0,
        )
        .unwrap()
    };

    (start, end)
}

fn time_to_range_index(time: NaiveTime, interval_size: usize) -> usize {
    if interval_size > 60 {
        panic!("Interval size cannot be greater than 60 minutes");
    }
    let minute = time.hour() * 60 + time.minute();
    (minute / interval_size as u32) as usize
}

impl DaySchedule {
    fn new() -> DaySchedule {
        let interval_size = 15; // default interval size is 15 minutes
        let blocks: Vec<TimeBlock> = (0..96)
            .map(|i| {
                let (start, end) = index_to_time_range(i, interval_size);
                TimeBlock {
                    start,
                    end,
                    block_type: BlockType::Free,
                }
            })
            .collect();

        DaySchedule {
            blocks,
            interval_size,
        }
    }
}

impl Default for WeekSchedule {
    fn default() -> Self {
        let mut days: HashMap<Weekday, DaySchedule> = HashMap::new();
        days.insert(Weekday::Mon, DaySchedule::new());
        days.insert(Weekday::Tue, DaySchedule::new());
        days.insert(Weekday::Wed, DaySchedule::new());
        days.insert(Weekday::Thu, DaySchedule::new());
        days.insert(Weekday::Fri, DaySchedule::new());
        days.insert(Weekday::Sat, DaySchedule::new());
        days.insert(Weekday::Sun, DaySchedule::new());

        WeekSchedule { days }
    }
}

#[test]
fn test_index_to_time_range() {
    assert_eq!(
        index_to_time_range(0, 15),
        (
            NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(0, 15, 0).unwrap()
        )
    );
    assert_eq!(
        index_to_time_range(9, 15),
        (
            NaiveTime::from_hms_opt(2, 15, 0).unwrap(),
            NaiveTime::from_hms_opt(2, 30, 0).unwrap()
        )
    );
    assert_eq!(
        index_to_time_range(95, 15),
        (
            NaiveTime::from_hms_opt(23, 45, 0).unwrap(),
            NaiveTime::from_hms_opt(11, 59, 59).unwrap()
        )
    );
}
