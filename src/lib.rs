use bitflags::bitflags;
use chrono::Datelike;

#[derive(Debug)]
pub enum RepeatingDateError {
    StartDateBeforeBound,
    WrongWeekday
}

#[derive(Debug)]
pub enum Repeating {
    Daily,
    Weekly,
    Monthly,
    Yearly
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
    pub struct WeekdayFlags: u8 {
        /// Monday.
        const MON = 1;
        /// Tuesday.
        const TUE = 2;
        /// Wednesday.
        const WED = 4;
        /// Thursday.
        const THU = 8;
        /// Friday.
        const FRI = 16;
        /// Saturday.
        const SAT = 32;
        /// Sunday.
        const SUN = 64;

        const ANY = WeekdayFlags::MON.bits() | WeekdayFlags::TUE.bits() | WeekdayFlags::WED.bits() 
            | WeekdayFlags::THU.bits() | WeekdayFlags::FRI.bits() | WeekdayFlags::SAT.bits() | WeekdayFlags::SUN.bits();

        const MIDWEEK = WeekdayFlags::MON.bits() | WeekdayFlags::TUE.bits() | WeekdayFlags::WED.bits() 
            | WeekdayFlags::THU.bits() | WeekdayFlags::FRI.bits();

        const WEEKEND = WeekdayFlags::SAT.bits() | WeekdayFlags::SUN.bits();
    }
}

impl WeekdayFlags {
    pub fn from_weekday(weekday: chrono::Weekday) -> WeekdayFlags {
        match weekday {
            chrono::Weekday::Mon => WeekdayFlags::MON,
            chrono::Weekday::Tue => WeekdayFlags::TUE,
            chrono::Weekday::Wed => WeekdayFlags::WED,
            chrono::Weekday::Thu => WeekdayFlags::THU,
            chrono::Weekday::Fri => WeekdayFlags::FRI,
            chrono::Weekday::Sat => WeekdayFlags::SAT,
            chrono::Weekday::Sun => WeekdayFlags::SUN,
        }
    }

    pub fn to_weekday(&self) -> chrono::Weekday {
        match *self {
            WeekdayFlags::MON => chrono::Weekday::Mon,
            WeekdayFlags::TUE => chrono::Weekday::Tue,
            WeekdayFlags::WED => chrono::Weekday::Wed,
            WeekdayFlags::THU => chrono::Weekday::Thu,
            WeekdayFlags::FRI => chrono::Weekday::Fri,
            WeekdayFlags::SAT => chrono::Weekday::Sat,
            WeekdayFlags::SUN => chrono::Weekday::Sun,
            _ => panic!("What ?")
        }
    }

    pub fn next_weekday(&self, current_dat: chrono::Weekday) -> chrono::Weekday {
        let weekday_flags: Vec<WeekdayFlags> = self.iter().collect();

        for i in &weekday_flags {
            let i_as_weekday = i.to_weekday();

            if i_as_weekday as u8 > current_dat as u8 {
                return i_as_weekday;
            }
        }
        
        weekday_flags[0].to_weekday()
    }

    pub fn next_weekday_bitwise(&self, current_dat: chrono::Weekday) -> chrono::Weekday {
        let mut weekday_flag = WeekdayFlags::from_weekday(current_dat).bits() << 1;
        let bits = self.bits();

        while weekday_flag > 0 && (bits & weekday_flag) == 0 {
            weekday_flag <<= 1;
        }
        
        if weekday_flag > 0 {
            WeekdayFlags::from_bits(weekday_flag).unwrap().to_weekday()
        } else {
            self.first_valid_weekday_bitwise()
        }
    }

    pub fn first_valid_weekday(&self, current_dat: chrono::Weekday) -> chrono::Weekday {
        for i in self.into_iter() {
            let i_as_weekday = i.to_weekday();

            if (i_as_weekday as u8) < (current_dat as u8) {
                return i_as_weekday;
            }
        }

        current_dat
    }

    pub fn first_valid_weekday_bitwise(&self) -> chrono::Weekday {
        let bits = self.bits();
        WeekdayFlags::from_bits(bits &  !( bits - 1 )).unwrap().to_weekday()
    }

    pub fn extract_weekdays(&self) -> Vec<chrono::Weekday> {
        let mut result = vec![];

        for i in self.iter() {
            match i {
                WeekdayFlags::MON => result.push(chrono::Weekday::Mon),
                WeekdayFlags::TUE => result.push(chrono::Weekday::Tue),
                WeekdayFlags::WED => result.push(chrono::Weekday::Wed),
                WeekdayFlags::THU => result.push(chrono::Weekday::Thu),
                WeekdayFlags::FRI => result.push(chrono::Weekday::Fri),
                WeekdayFlags::SAT => result.push(chrono::Weekday::Sat),
                WeekdayFlags::SUN => result.push(chrono::Weekday::Sun),
                _ => panic!("What ?")
            }
        }

        result
    }
}

pub fn days_until(current_dat: chrono::Weekday, next_dat: chrono::Weekday) -> i32 {
    let weekday_diff = (next_dat as i8) - (current_dat as i8);

    ((7 + weekday_diff) % 7) as i32
}

pub fn get_months_since(from_date: chrono::NaiveDate, start_date: chrono::NaiveDate) -> i32 {
    let years_diff = from_date.year_ce().1 as i32 - start_date.year_ce().1 as i32;

    let years_months = years_diff * 12;
    
    from_date.month() as i32 - start_date.month() as i32 + years_months
}

pub fn find_next_weekstart(from_date: chrono::NaiveDate, start_date: chrono::NaiveDate, weekdays: WeekdayFlags, interval: u64) -> chrono::NaiveDate {
    let date_diff = from_date - start_date;
    let date_delta_days = date_diff.num_days() as u64;

    let is_any_weekday_valid = weekdays.is_all();

    let days_needed_weekly = interval * 7;
    let days_needed_total = days_needed_weekly + date_delta_days;
    let fix = days_needed_total % days_needed_weekly;

    let day_offset = days_needed_total - fix;

    let date = start_date.checked_add_days(chrono::Days::new(day_offset)).unwrap();
    let result_weekday = date.weekday();

    let previous_weekday = weekdays.first_valid_weekday_bitwise();
    let weekdays_offset_abs = ((!is_any_weekday_valid) as u8) * ((result_weekday as u8) - (previous_weekday as u8));

    date.checked_sub_days(chrono::Days::new(weekdays_offset_abs as u64)).unwrap()
}

pub fn find_next_date(from_date: chrono::NaiveDate, start_date: chrono::NaiveDate, weekdays: WeekdayFlags, repeat: Repeating, interval: u64) -> Result<chrono::NaiveDate, RepeatingDateError> {
    if from_date < start_date {
        return Err(RepeatingDateError::StartDateBeforeBound);
    }
    
    if !weekdays.contains(WeekdayFlags::from_weekday(start_date.weekday())) {
        return Err(RepeatingDateError::WrongWeekday);
    }

    let date_diff = from_date - start_date;
    let date_delta_days = date_diff.num_days() as u64;

    match repeat {
        Repeating::Daily => {
            let date_delta_mod = date_delta_days % interval;

            let mut date = start_date.checked_add_days(chrono::Days::new(date_delta_mod + date_delta_days + interval)).unwrap();

            while !weekdays.contains(WeekdayFlags::from_weekday(date.weekday())) {
                date = date.checked_add_days(chrono::Days::new(interval)).unwrap();
            }

            Ok(date)
        },
        Repeating::Weekly => {
            let next_weekday = weekdays.next_weekday_bitwise(from_date.weekday());
            let from_date_weekday = from_date.weekday();

            let is_any_weekday_valid = weekdays.is_all();

            let days_until_next_valid_weekday = ((!is_any_weekday_valid) as i32) * days_until(from_date_weekday, next_weekday);

            let date = from_date.checked_add_days(chrono::Days::new(days_until_next_valid_weekday as u64)).unwrap();

            if date > from_date && match_repeating_date(date, start_date, weekdays, repeat, interval) {
                return Ok(date);
            }

            Ok(find_next_weekstart(from_date, start_date, weekdays, interval))
        },
        Repeating::Monthly => {
            let interval = interval as i32;
            let month_diff = get_months_since(from_date, start_date);

            let delta = interval + (((month_diff % interval == 0) as i32) * month_diff);

            let mut date = start_date.checked_add_months(chrono::Months::new(delta as u32)).unwrap();

            while !weekdays.contains(WeekdayFlags::from_weekday(date.weekday())) {
                date = date.checked_add_months(chrono::Months::new(interval as u32)).unwrap();
            }

            Ok(date)
        },
        Repeating::Yearly => {
            let max_year_skip = 100;

            let interval = interval as i32;
            let years_delta = from_date.years_since(start_date).unwrap() as i32;

            let (_, current_year) = start_date.year_ce();
            let mut date = start_date.with_year(current_year as i32 + years_delta + interval).unwrap();

            let mut counter = 0;
            while counter < max_year_skip && !weekdays.contains(WeekdayFlags::from_weekday(date.weekday())) {
                let (_, cur_year) = start_date.year_ce();
                date = date.with_year(cur_year as i32 + interval).unwrap();

                counter += 1;
            }

            Ok(date)
        },
    }
}

pub fn match_repeating_date(date_to_check: chrono::NaiveDate, start_date: chrono::NaiveDate, weekdays: WeekdayFlags, repeat: Repeating, interval: u64) -> bool {
    if date_to_check < start_date {
        return false;
    }
    
    if !weekdays.contains(WeekdayFlags::from_weekday(date_to_check.weekday())) {
        return false;
    }

    let date_diff = date_to_check - start_date;

    match repeat {
        Repeating::Daily => date_diff.num_days() % interval as i64 == 0,
        Repeating::Weekly => {
            let date_to_check_from_monday = date_to_check.weekday().number_from_monday();
            let start_date_from_monday = start_date.weekday().number_from_monday();

            let is_new_week = date_to_check_from_monday < start_date_from_monday;

            (date_diff.num_weeks() + (is_new_week as i64)) % interval as i64 == 0
        },
        Repeating::Monthly => {
            let month_diff = get_months_since(date_to_check, start_date);

            date_to_check.day0() == start_date.day0() 
                && month_diff > 0 && month_diff % interval as i32 == 0
        },
        Repeating::Yearly => if let Some(years) = date_to_check.years_since(start_date) {
            date_to_check.day0() == start_date.day0() && date_to_check.month() == start_date.month()
                && years > 0 && years % interval as u32 == 0
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;
    use rstest::rstest;
    use chrono::Datelike;

    #[test]
    fn daily_schedule_10_repeats() {
        let limit = 10;
        let weekdays = WeekdayFlags::MIDWEEK;
        let start_date = chrono::NaiveDate::from_str("2023-09-18").unwrap();
        let dates_in_range = ["2023-09-19", "2023-09-20", "2023-09-21", "2023-09-22", "2023-09-25", "2023-09-26", "2023-09-27", "2023-09-28", "2023-09-29"];

        let mut counter = 1;
        let mut result = start_date;
        for expected_date_string in dates_in_range {
            result = find_next_date(result, start_date, weekdays, Repeating::Daily, 1).unwrap();

            let expected_result = chrono::NaiveDate::from_str(expected_date_string).unwrap();
            assert_eq!(expected_result, result);
            assert!(weekdays.contains(WeekdayFlags::from_weekday(result.weekday())));
            assert!(match_repeating_date(result, start_date, weekdays, Repeating::Daily, 1));

            counter += 1;
        }

        assert_eq!(limit, counter);
    }

    #[test]
    fn bi_daily_schedule_5_repeats() {
        let limit = 5;
        let weekdays = WeekdayFlags::MIDWEEK;
        let start_date = chrono::NaiveDate::from_str("2023-09-18").unwrap();
        let dates_in_range = ["2023-09-20", "2023-09-22", "2023-09-26", "2023-09-28"];

        let mut counter = 1;
        let mut result = start_date;
        for expected_date_string in dates_in_range {
            result = find_next_date(result, start_date, weekdays,  Repeating::Daily, 2).unwrap();

            let expected_result = chrono::NaiveDate::from_str(expected_date_string).unwrap();
            assert_eq!(expected_result, result);
            assert!(weekdays.contains(WeekdayFlags::from_weekday(result.weekday())));
            assert!(match_repeating_date(result, start_date, weekdays, Repeating::Daily, 2));

            counter += 1;
        }

        assert_eq!(limit, counter);
    }

    #[test]
    fn weekly_schedule_14_repeats() {
        let limit = 14;
        let weekdays = WeekdayFlags::THU;
        let start_date = chrono::NaiveDate::from_str("2023-09-21").unwrap();
        let dates_in_range = ["2023-09-28", "2023-10-05", "2023-10-12", "2023-10-19", "2023-10-26", "2023-11-02", "2023-11-09", "2023-11-16", "2023-11-23", "2023-11-30", "2023-12-07", "2023-12-14", "2023-12-21"];

        let mut counter = 1;
        let mut result = start_date;
        for expected_date_string in dates_in_range {
            result = find_next_date(result, start_date, weekdays, Repeating::Weekly, 1).unwrap();

            let expected_result = chrono::NaiveDate::from_str(expected_date_string).unwrap();
            assert_eq!(expected_result, result);
            assert!(weekdays.contains(WeekdayFlags::from_weekday(result.weekday())));
            assert!(match_repeating_date(result, start_date, weekdays, Repeating::Weekly, 1));

            counter += 1;
        }

        assert_eq!(limit, counter);
    }

    #[test]
    fn bi_weekly_schedule_7_repeats() {
        let limit = 7;
        let weekdays = WeekdayFlags::THU;
        let start_date = chrono::NaiveDate::from_str("2023-09-21").unwrap();
        let dates_in_range = ["2023-10-05", "2023-10-19", "2023-11-02", "2023-11-16", "2023-11-30", "2023-12-14"];

        let mut counter = 1;
        let mut result = start_date;
        for expected_date_string in dates_in_range {
            result = find_next_date(result, start_date, weekdays, Repeating::Weekly, 2).unwrap();

            let expected_result = chrono::NaiveDate::from_str(expected_date_string).unwrap();
            assert_eq!(expected_result, result);
            assert!(weekdays.contains(WeekdayFlags::from_weekday(result.weekday())));
            assert!(match_repeating_date(result, start_date, weekdays, Repeating::Weekly, 2));

            counter += 1;
        }

        assert_eq!(limit, counter);
    }

    #[test]
    fn weekly_schedule_15_repeats_moredays() {
        let limit = 15;
        let weekdays = WeekdayFlags::MON | WeekdayFlags::WED | WeekdayFlags::FRI | WeekdayFlags::SAT;
        let start_date = chrono::NaiveDate::from_str("2023-10-11").unwrap();
        let dates_in_range = ["2023-10-13", "2023-10-14", "2023-10-16", "2023-10-18", "2023-10-20", "2023-10-21", "2023-10-23", "2023-10-25", "2023-10-27", "2023-10-28", "2023-10-30", "2023-11-01", "2023-11-03", "2023-11-04"];

        let mut counter = 1;
        let mut result = start_date;
        for expected_date_string in dates_in_range {
            result = find_next_date(result, start_date, weekdays, Repeating::Weekly, 1).unwrap();

            let expected_result = chrono::NaiveDate::from_str(expected_date_string).unwrap();
            assert_eq!(expected_result, result);
            assert!(weekdays.contains(WeekdayFlags::from_weekday(result.weekday())));
            assert!(match_repeating_date(result, start_date, weekdays, Repeating::Weekly, 1));

            counter += 1;
        }

        assert_eq!(limit, counter);
    }

    #[test]
    fn weekly_schedule_3_weeks_moredays() {
        let limit = 5;
        let weekdays = WeekdayFlags::MON | WeekdayFlags::WED | WeekdayFlags::FRI | WeekdayFlags::SAT;
        let start_date = chrono::NaiveDate::from_str("2023-10-11").unwrap();
        let dates_in_range = ["2023-10-13", "2023-10-14", "2023-10-30", "2023-11-01"];

        let mut counter = 1;
        let mut result = start_date;
        for expected_date_string in dates_in_range {
            result = find_next_date(result, start_date, weekdays, Repeating::Weekly, 3).unwrap();

            let expected_result = chrono::NaiveDate::from_str(expected_date_string).unwrap();
            assert_eq!(expected_result, result);
            assert!(weekdays.contains(WeekdayFlags::from_weekday(result.weekday())));
            assert!(match_repeating_date(result, start_date, weekdays, Repeating::Weekly, 3));

            counter += 1;
        }

        assert_eq!(limit, counter);
    }

    #[test]
    fn monthly_schedule_until_date() {
        let limit = 6;
        let weekdays = WeekdayFlags::ANY;
        let start_date = chrono::NaiveDate::from_str("2023-09-19").unwrap();
        let dates_in_range = ["2023-10-19", "2023-11-19", "2023-12-19", "2024-01-19", "2024-02-19"];
        let final_date = chrono::NaiveDate::from_str("2024-02-19").unwrap();

        let mut counter = 0;
        let mut result = start_date;
        while result < final_date {
            result = find_next_date(result, start_date, weekdays, Repeating::Monthly, 1).unwrap();

            let expected_result = chrono::NaiveDate::from_str(dates_in_range[counter]).unwrap();
            assert_eq!(expected_result, result);
            assert!(weekdays.contains(WeekdayFlags::from_weekday(result.weekday())));
            assert!(match_repeating_date(result, start_date, weekdays, Repeating::Monthly, 1));

            counter += 1;
        }

        assert_eq!(limit, counter + 1);
    }

    #[test]
    fn bi_monthly_schedule_until_date() {
        let limit = 6;
        let weekdays = WeekdayFlags::ANY;
        let start_date = chrono::NaiveDate::from_str("2023-09-19").unwrap();
        let dates_in_range = ["2023-11-19", "2024-01-19", "2024-03-19", "2024-05-19", "2024-07-19"];
        let final_date = chrono::NaiveDate::from_str("2024-06-19").unwrap();

        let mut counter = 0;
        let mut result = start_date;
        while result < final_date {
            result = find_next_date(result, start_date, weekdays, Repeating::Monthly, 2).unwrap();

            let expected_result = chrono::NaiveDate::from_str(dates_in_range[counter]).unwrap();
            assert_eq!(expected_result, result);
            assert!(weekdays.contains(WeekdayFlags::from_weekday(result.weekday())));
            assert!(match_repeating_date(result, start_date, weekdays, Repeating::Monthly, 2));

            counter += 1;
        }

        assert_eq!(limit, counter + 1);
    }

    #[rstest]
    #[case::same_week("2023-9-12", "2023-9-12", WeekdayFlags::TUE | WeekdayFlags::THU, 2, "2023-9-14")]
    #[case::same_week("2023-9-12", "2023-9-14", WeekdayFlags::TUE | WeekdayFlags::THU, 2, "2023-9-26")]
    #[case::same_week("2023-9-12", "2023-9-14", WeekdayFlags::TUE | WeekdayFlags::THU | WeekdayFlags::FRI, 2, "2023-9-22")]
    #[case("2023-9-12", "2023-9-12", WeekdayFlags::TUE | WeekdayFlags::THU, 1, "2023-9-14")]
    #[case::three_days("2023-9-12", "2023-9-21", WeekdayFlags::TUE | WeekdayFlags::THU, 3, "2023-10-3")]
    fn next_daily(#[case] start: chrono::NaiveDate, #[case] from: chrono::NaiveDate, #[case] weekdays: WeekdayFlags, 
        #[case] interval: u64, #[case] expected_result: chrono::NaiveDate) {
        let result = find_next_date(from, start, weekdays, Repeating::Daily, interval).unwrap();

        assert_eq!(expected_result, result);
        assert!(weekdays.contains(WeekdayFlags::from_weekday(result.weekday())));
        assert!(match_repeating_date(result, start, weekdays, Repeating::Daily, interval));
    }

    #[rstest]
    #[case::same_week_two_days("2023-9-12", "2023-9-12", WeekdayFlags::TUE | WeekdayFlags::THU, 2, "2023-9-14")]
    #[case::same_week_two_days("2023-9-12", "2023-9-14", WeekdayFlags::TUE | WeekdayFlags::THU, 2, "2023-9-26")]
    #[case::same_week("2023-9-12", "2023-9-12", WeekdayFlags::TUE | WeekdayFlags::THU, 1, "2023-9-14")]
    #[case::same_week("2023-9-12", "2023-9-13", WeekdayFlags::TUE | WeekdayFlags::THU, 1, "2023-9-14")]
    #[case::same_week_one_day("2023-9-12", "2023-9-13", WeekdayFlags::TUE, 1, "2023-9-19")]
    #[case::three_week_same_week("2023-9-12", "2023-9-14", WeekdayFlags::TUE | WeekdayFlags::THU, 3, "2023-10-3")]
    #[case::three_week("2023-9-12", "2023-9-21", WeekdayFlags::TUE | WeekdayFlags::THU, 3, "2023-10-3")]
    #[case::three_week("2023-9-12", "2023-9-20", WeekdayFlags::TUE | WeekdayFlags::THU, 3, "2023-10-3")]
    #[case::two_week("2023-9-12", "2023-9-21", WeekdayFlags::TUE | WeekdayFlags::THU, 2, "2023-9-26")]
    #[case::two_week_different_month("2023-9-12", "2023-11-30", WeekdayFlags::TUE | WeekdayFlags::THU, 2, "2023-12-5")]
    #[case::two_week_same_week("2023-9-12", "2023-9-20", WeekdayFlags::TUE | WeekdayFlags::THU, 2, "2023-9-26")]
    #[case::three_days("2023-9-12", "2023-9-14", WeekdayFlags::TUE | WeekdayFlags::THU | WeekdayFlags::FRI, 2, "2023-9-15")]
    fn next_weekly(#[case] start: chrono::NaiveDate, #[case] from: chrono::NaiveDate, #[case] weekdays: WeekdayFlags, 
        #[case] interval: u64, #[case] expected_result: chrono::NaiveDate) {
        let result = find_next_date(from, start, weekdays, Repeating::Weekly, interval).unwrap();

        assert_eq!(expected_result, result);
        assert!(weekdays.contains(WeekdayFlags::from_weekday(result.weekday())));
        assert!(match_repeating_date(result, start, weekdays, Repeating::Weekly, interval));
    }

    #[rstest]
    #[case("2023-9-12", "2023-9-12", WeekdayFlags::TUE | WeekdayFlags::THU, 1, "2023-10-12")]
    #[case("2023-9-12", "2023-9-12", WeekdayFlags::TUE | WeekdayFlags::THU | WeekdayFlags::SUN, 2, "2023-11-12")]
    #[case("2023-9-12", "2023-10-12", WeekdayFlags::TUE | WeekdayFlags::THU, 2, "2024-03-12")]
    #[case("2023-9-12", "2024-03-12", WeekdayFlags::TUE | WeekdayFlags::THU, 2, "2024-09-12")]
    #[case::diff_3_months("2023-9-12", "2023-12-12", WeekdayFlags::TUE | WeekdayFlags::THU, 2, "2024-03-12")]
    #[case::skip_5("2023-9-12", "2023-12-12", WeekdayFlags::TUE | WeekdayFlags::THU, 5, "2024-12-12")]
    fn next_monthly(#[case] start: chrono::NaiveDate, #[case] from: chrono::NaiveDate, #[case] weekdays: WeekdayFlags,
        #[case] interval: u64, #[case] expected_result: chrono::NaiveDate) {
        let result = find_next_date(from, start, weekdays, Repeating::Monthly, interval).unwrap();

        assert_eq!(expected_result, result);
        assert!(weekdays.contains(WeekdayFlags::from_weekday(result.weekday())));
        assert!(match_repeating_date(result, start, weekdays, Repeating::Monthly, interval));
    }

    #[rstest]
    #[case("2023-9-12", "2023-9-12", WeekdayFlags::TUE | WeekdayFlags::THU, 1, "2024-9-12")]
    #[case("2023-9-12", "2023-9-12", WeekdayFlags::TUE | WeekdayFlags::THU | WeekdayFlags::FRI, 2, "2025-9-12")]
    #[case("2023-9-12", "2023-9-12", WeekdayFlags::ANY, 2, "2025-9-12")]
    #[case("2023-9-12", "2023-12-12", WeekdayFlags::ANY, 3, "2026-9-12")]
    fn next_yearly(#[case] start: chrono::NaiveDate, #[case] from: chrono::NaiveDate, #[case] weekdays: WeekdayFlags, 
        #[case] interval: u64, #[case] expected_result: chrono::NaiveDate) {
        let result = find_next_date(from, start, weekdays, Repeating::Yearly, interval).unwrap();

        assert_eq!(expected_result, result);
        assert!(weekdays.contains(WeekdayFlags::from_weekday(result.weekday())));
        assert!(match_repeating_date(result, start, weekdays, Repeating::Yearly, interval));
    }

    #[rstest]
    #[case::two_week("2023-9-26", "2023-9-12", WeekdayFlags::TUE | WeekdayFlags::THU, 2, true)]
    #[case::same_week("2023-9-14", "2023-9-12", WeekdayFlags::TUE | WeekdayFlags::THU, 2, true)]
    #[case::same_week_different_day("2023-9-15", "2023-9-12", WeekdayFlags::TUE | WeekdayFlags::THU, 2, false)]
    #[case::one_week("2023-9-19", "2023-9-12", WeekdayFlags::TUE | WeekdayFlags::THU, 1, true)]
    #[case::three_weeks("2023-9-26", "2023-9-12", WeekdayFlags::TUE | WeekdayFlags::THU, 3, false)]
    #[case::different_day("2023-9-26", "2023-9-12", WeekdayFlags::MON, 3, false)]
    fn weekly_repeat_match(#[case] check: chrono::NaiveDate, #[case] start: chrono::NaiveDate, #[case] weekdays: WeekdayFlags,
        #[case] interval: u64, #[case] expected_result: bool) {
        let result = match_repeating_date(check, start, weekdays, Repeating::Weekly, interval);

        assert_eq!(expected_result, result);
    }

    #[rstest]
    #[case("2023-9-13", "2023-9-12", WeekdayFlags::TUE | WeekdayFlags::WED, 1, true)]
    #[case::every_two_days("2023-9-14", "2023-9-12", WeekdayFlags::TUE | WeekdayFlags::THU, 2, true)]
    #[case::different_month("2023-10-3", "2023-9-12", WeekdayFlags::TUE | WeekdayFlags::WED, 1, true)]
    #[case::wrong_day("2023-10-2", "2023-9-12", WeekdayFlags::TUE | WeekdayFlags::WED, 1, false)]
    #[case::every_ten_days("2023-9-22", "2023-9-12", WeekdayFlags::TUE | WeekdayFlags::FRI, 10, true)]
    #[case::every_ten_days("2023-10-2", "2023-9-12", WeekdayFlags::MON | WeekdayFlags::FRI, 10, true)]
    fn daily_repeat_match(#[case] check: chrono::NaiveDate, #[case] start: chrono::NaiveDate, #[case] weekdays: WeekdayFlags,
        #[case] interval: u64, #[case] expected_result: bool) {
        let result = match_repeating_date(check, start, weekdays, Repeating::Daily, interval);

        assert_eq!(expected_result, result);
    }

    #[rstest]
    #[case::same_month("2023-9-14", "2023-9-12", 2, false)]
    #[case::same_month("2023-9-14", "2023-9-12", 1, false)]
    #[case::every_two_months("2023-11-12", "2023-9-12", 2, true)]
    #[case::wrong_day("2023-11-13", "2023-9-12", 1, false)]
    #[case::every_month("2023-10-12", "2023-9-12", 1, true)]
    #[case::every_month("2023-12-12", "2023-9-12", 1, true)]
    #[case::wrong_month("2023-10-12", "2023-9-12", 2, false)]
    fn monthly_repeat_match(#[case] check: chrono::NaiveDate, #[case] start: chrono::NaiveDate, #[case] interval: u64, #[case] expected_result: bool) {
        let result = match_repeating_date(check, start, WeekdayFlags::ANY, Repeating::Monthly, interval);

        assert_eq!(expected_result, result);
    }

    #[rstest]
    #[case::same_year("2023-9-14", "2023-9-12", 1, false)]
    #[case::every_two_year("2025-9-12", "2023-9-12", 2, true)]
    #[case::wrong_day("2024-11-13", "2023-9-12", 1, false)]
    #[case::every_year("2024-9-12", "2023-9-12", 1, true)]
    #[case::every_year("2025-9-12", "2023-9-12", 1, true)]
    #[case::wrong_year("2024-10-12", "2023-9-12", 2, false)]
    fn yearly_repeat_match(#[case] check: chrono::NaiveDate, #[case] start: chrono::NaiveDate, #[case] interval: u64, #[case] expected_result: bool) {
        let result = match_repeating_date(check, start, WeekdayFlags::ANY, Repeating::Yearly, interval);

        assert_eq!(expected_result, result);
    }
}
