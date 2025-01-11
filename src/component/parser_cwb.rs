use std::{error::Error, str::Split};

use chrono::{NaiveDateTime, NaiveTime};

type INTEGER = i64;
type FLOAT = f64;

fn parse_string(spliter: &mut Split<char>) -> Result<String, Box<dyn Error + 'static>> {
    if let Some(val) = spliter.next() {
        log::debug!("{val}");
        Ok(val.to_string())
    } else {
        Err(String::from("System error").into())
    }
}

fn parse_integer(spliter: &mut Split<char>) -> Result<INTEGER, Box<dyn Error + 'static>> {
    if let Some(val) = spliter.next() {
        log::debug!("{val}");
        Ok(val.parse::<INTEGER>()?)
    } else {
        Err(String::from("System error").into())
    }
}

fn parse_float(spliter: &mut Split<char>) -> Result<FLOAT, Box<dyn Error + 'static>> {
    if let Some(val) = spliter.next() {
        log::debug!("{val}");
        Ok(val.parse::<FLOAT>()?)
    } else {
        Err(String::from("System error").into())
    }
}

fn parse_datetime(spliter: &mut Split<char>) -> Result<NaiveDateTime, Box<dyn Error + 'static>> {
    if let Some(val) = spliter.next() {
        log::debug!("{val}");
        Ok(NaiveDateTime::parse_from_str(val, "%Y%m%d%H%M")?)
    } else {
        Err(String::from("System error").into())
    }
}

fn parse_time(spliter: &mut Split<char>) -> Result<NaiveTime, Box<dyn Error + 'static>> {
    if let Some(val) = spliter.next() {
        log::debug!("{val}");
        Ok(NaiveTime::parse_from_str(val, "%H%M")?)
    } else {
        Err(String::from("System error").into())
    }
}

#[derive(Debug, Default)]
pub struct CWBMinData {
    station: String,
    dkind: String,
    dtime: NaiveDateTime, // local time
    dd_p1: FLOAT,         // hPa, station pressure
    dd_mmp2: FLOAT,       // hPa, sea level pressure
    dd_t: FLOAT,          // degC, temperature
    dd_td: FLOAT,         // degC, dew point temperature
    dd_rh: FLOAT,         // %, relative humidity
    dd_e: FLOAT,          // hPa, staturated vapor
    dd_e_: FLOAT,         // hPa, vapor
    dd_10d: INTEGER,      // vector, 10 min average wind direction
    dd_f10: FLOAT,        // m/s, 10 min average wind speed
    dd_xxd: INTEGER,      // vector, wind direction at max wind speed
    dd_fxx: FLOAT,        // m/s, max wind speed
    dd_rmn: FLOAT,        // mm, rain per min
    dd_r: FLOAT,          // mm, rain per hour
    dd_p1x: FLOAT,        // hPa, daily max pressure
    ddp1xt: NaiveTime,    // local time, time at daily max pressure
    dd_p1n: FLOAT,        // hPa, daily min pressure
    ddp1nt: NaiveTime,    // local time, time at daily min pressure
    dd_tx: FLOAT,         // degC, daily max temperature
    dd_txt: NaiveTime,    // local time, time at daily max temperature
    dd_tn: FLOAT,         // degC, daily min temperature
    dd_tnt: NaiveTime,    // local time, time at daily min temperature
    dd_tdx: FLOAT,        // degC, daily max dew point temperature
    ddtdxt: NaiveTime,    // local time, time at daily max dew point temperature
    dd_tdn: FLOAT,        // degC, daily min dew point temperature
    ddtdnt: NaiveTime,    // local time, time at daily min dew point temperature
    dd_rhx: FLOAT,        // %, daily max relative humidity
    ddrhxt: NaiveTime,    // local time, time at daily max relative humidity
    dd_rhn: FLOAT,        // %, daily min relative humidity
    ddrhnt: NaiveTime,    // local time, time at daily min relative humidity
    dd_ex: FLOAT,         // hPa, daily max vapor
    dd_ext: NaiveTime,    // local time, time at daily max vapor
    dd_en: FLOAT,         // %, daily min vapor
    dd_ent: NaiveTime,    // local time, time at daily min vapor
    dd_fx: FLOAT,         // m/s, daily max wind speed
    dd_xd: INTEGER,       // vector, wind direction at daily max wind speed
    dd_fxt: FLOAT,        // local time, time at daily max wind speed
    dd_f10x: FLOAT,       // m/s, [dd_f10] daily max 10 min average wind speed
    dd_10dx: INTEGER,     // vector, [dd_10d] wind direction at daily max 10 min average wind speed
    ddf10xt: NaiveTime,   // loacl time, [ddf10xt] time at daily max 10 min average wind speed
    dd_wd: FLOAT,         // m, average wind range
    dd_tr: FLOAT,         // mm, daily rain
    dd_rx10: FLOAT,       // mm, max rain with 10 min (10 分鐘最大累積雨量)
    dd_x10t: NaiveTime,   // local time, time at max rain with 10 min
    dd_rx60: FLOAT,       // mm, max rain with 60 min (60 分鐘最大累積雨量)
    dd_x60t: NaiveTime,   // local time , time at max rain with 60 min
    dd_tgr: FLOAT,        // MJ/m2, daily accumulation of solar radiation
    dd_rad: FLOAT,        // KJ/m2, accumulation of solar radiation per min
    dd_gr: FLOAT,         // MJ/m2, accumulation of solar radiaiton per hour
    dd_tsh: FLOAT,        // Hr, daily accumulation of sunshine
    dd_sh: FLOAT,         // Hr, accumulation of sunshine per hour
    dd_t05: FLOAT,        // degC, 5 cm grassland temperature
    dd_t00: FLOAT,        // degC, 0 cm grassland temperature
    dd_st005: FLOAT,      // degC, 5 cm under ground temperature
    dd_st010: FLOAT,      // degC, 10 cm under ground temperature
    dd_st020: FLOAT,      // degC, 20 cm under ground temperature
    dd_st030: FLOAT,      // degC, 30 cm under ground temperature
    dd_st050: FLOAT,      // degC, 50 cm under ground temperature
    dd_st100: FLOAT,      // degC, 100 cm under ground temperature
    dd_sitest: String,    // station status code
}

impl CWBMinData {
    pub fn parse_from_str(data: &str) -> Result<Self, Box<dyn Error + 'static>> {
        let mut words = data.split(',');
        if words.nth(2) != Some("MN") {
            return Err(String::from("Invalid").into());
        }

        let mut words = data.split(',');

        // stx
        if words.next() != Some("\u{2}") {
            return Err(String::from("Invalid").into());
        }

        let result = CWBMinData {
            station: parse_string(&mut words)?,
            dkind: parse_string(&mut words)?,
            dtime: parse_datetime(&mut words)?,
            dd_p1: parse_float(&mut words)?,
            dd_mmp2: parse_float(&mut words)?,
            dd_t: parse_float(&mut words)?,
            dd_td: parse_float(&mut words)?,
            dd_rh: parse_float(&mut words)?,
            dd_e: parse_float(&mut words)?,
            dd_e_: parse_float(&mut words)?,
            dd_10d: parse_integer(&mut words)?,
            dd_f10: parse_float(&mut words)?,
            dd_xxd: parse_integer(&mut words)?,
            dd_fxx: parse_float(&mut words)?,
            dd_rmn: parse_float(&mut words)?,
            dd_r: parse_float(&mut words)?,
            dd_p1x: parse_float(&mut words)?,
            ddp1xt: parse_time(&mut words)?,
            dd_p1n: parse_float(&mut words)?,
            ddp1nt: parse_time(&mut words)?,
            dd_tx: parse_float(&mut words)?,
            dd_txt: parse_time(&mut words)?,
            dd_tn: parse_float(&mut words)?,
            dd_tnt: parse_time(&mut words)?,
            dd_tdx: parse_float(&mut words)?,
            ddtdxt: parse_time(&mut words)?,
            dd_tdn: parse_float(&mut words)?,
            ddtdnt: parse_time(&mut words)?,
            dd_rhx: parse_float(&mut words)?,
            ddrhxt: parse_time(&mut words)?,
            dd_rhn: parse_float(&mut words)?,
            ddrhnt: parse_time(&mut words)?,
            dd_ex: parse_float(&mut words)?,
            dd_ext: parse_time(&mut words)?,
            dd_en: parse_float(&mut words)?,
            dd_ent: parse_time(&mut words)?,
            dd_fx: parse_float(&mut words)?,
            dd_xd: parse_integer(&mut words)?,
            dd_fxt: parse_float(&mut words)?,
            dd_f10x: parse_float(&mut words)?,
            dd_10dx: parse_integer(&mut words)?,
            ddf10xt: parse_time(&mut words)?,
            dd_wd: parse_float(&mut words)?,
            dd_tr: parse_float(&mut words)?,
            dd_rx10: parse_float(&mut words)?,
            dd_x10t: parse_time(&mut words)?,
            dd_rx60: parse_float(&mut words)?,
            dd_x60t: parse_time(&mut words)?,
            dd_tgr: parse_float(&mut words)?,
            dd_rad: parse_float(&mut words)?,
            dd_gr: parse_float(&mut words)?,
            dd_tsh: parse_float(&mut words)?,
            dd_sh: parse_float(&mut words)?,
            dd_t05: parse_float(&mut words)?,
            dd_t00: parse_float(&mut words)?,
            dd_st005: parse_float(&mut words)?,
            dd_st010: parse_float(&mut words)?,
            dd_st020: parse_float(&mut words)?,
            dd_st030: parse_float(&mut words)?,
            dd_st050: parse_float(&mut words)?,
            dd_st100: parse_float(&mut words)?,
            dd_sitest: parse_string(&mut words)?,
        };

        // etx
        if words.next() != Some("\u{3}") {
            return Err(String::from("Invalid").into());
        }

        Ok(result)
    }
}

#[derive(Debug, Default)]
pub struct CWBHourData {
    station: String,
    dkind: String,
    dtime: NaiveDateTime, // local time
    h_p1: FLOAT,          // hPa, station pressure
    h_p2: FLOAT,          // hPa, sea level pressure
    h_a: INTEGER,         // pressure tendency
    h_pp: FLOAT,          // 3 hour pressure diff
    h_e: FLOAT,           // hPa, staturated vapor
    h_e_: FLOAT,          // hPa, vapor
    h_t: FLOAT,           // degC, temperature
    h_tx: FLOAT,          // degC, daily max temperature
    h_tn: FLOAT,          // degC, daily min temperature
    h_td: FLOAT,          // degC, dew point temperature
    h_rh: FLOAT,          // %, relative humidity
    h_dd: INTEGER,        // vector, average wind direction
    h_ff: FLOAT,          // m/s, average wind speed
    h_f10: FLOAT,         // m/s, max average wind speed
    h_10d: INTEGER,       // vector, wind direction at max average wind speed
    h_f10t: NaiveTime,    // local time, time at max average wind speed
    h_fx: FLOAT,          // m/s, max wind speed
    h_xd: INTEGER,        // vector, wind direction at max wind speed
    h_fxt: NaiveTime,     // local time, time at max wind speed
    h_xd3: INTEGER,       // vector, wind direction at 3 hour max wind speed
    h_fx3: FLOAT,         // m/s, 3 hour max wind speed
    h_r: FLOAT,           // mm, rain per hour
    h_gmt: FLOAT,         // mm, gmt rain
    h_3r: FLOAT,          // mm, rain in 3 hours
    h_6r: FLOAT,          // mm, rain in 6 hours
    h_9r: FLOAT,          // mm, rain in 9 hours
    h_12r: FLOAT,         // mm, rain in 12 hours
    h_24r: FLOAT,         // mm, rain in 24 hours in local time
    h_24rrr: FLOAT,       // mm, yesterday rain
    h_gr: FLOAT,          // MJ/m2, accumulation of solar radiaiton per hour
    h_sh: FLOAT,          // Hr, accumulation of sunshine per hour
    h_t005: FLOAT,        // degC, 5 cm grassland temperature
    h_t005x: FLOAT,       // degC, 5 cm grassland max temperature
    h_t005xt: NaiveTime,  // local time, time at 5 cm grassland max temperature
    h_t005n: FLOAT,       // degC, 5 cm grassland min temperature
    h_t005nt: NaiveTime,  // local time, time at 5 cm grassland min temperature
    h_t000: FLOAT,        // degC, 0 cm grassland temperature
    h_t000x: FLOAT,       // degC, 0 cm grassland max temperature
    h_t000xt: NaiveTime,  // local time, time at 0 cm grassland max temperature
    h_t000n: FLOAT,       // degC, 0 cm grassland min temperature
    h_t000nt: NaiveTime,  // local time, time at 0 cm grassland min temperature
    h_st005: FLOAT,       // degC, 5 cm under ground temperature
    h_st005x: FLOAT,      // degC, 5 cm under ground max temperature
    h_st005xt: NaiveTime, // local time, time at 5 cm under ground max temperature
    h_st005n: FLOAT,      // degC, 5 cm under ground min temperature
    h_st005nt: NaiveTime, // local time, time at 5 cm under ground min temperature
    h_st010: FLOAT,       // degC, 10 cm under ground temperature
    h_st010x: FLOAT,      // degC, 10 cm under ground max temperature
    h_st010xt: NaiveTime, // local time, time at 10 cm under ground max temperature
    h_st010n: FLOAT,      // degC, 10 cm under ground min temperature
    h_st010nt: NaiveTime, // local time, time at 10 cm under ground min temperature
    h_st020: FLOAT,       // degC, 20 cm under ground temperature
    h_st020x: FLOAT,      // degC, 20 cm under ground max temperature
    h_st020xt: NaiveTime, // local time, time at 20 cm under ground max temperature
    h_st020n: FLOAT,      // degC, 20 cm under ground min temperature
    h_st020nt: NaiveTime, // local time, time at 20 cm under ground min temperature
    h_st030: FLOAT,       // degC, 30 cm under ground temperature
    h_st030x: FLOAT,      // degC, 30 cm under ground max temperature
    h_st030xt: NaiveTime, // local time, time at 30 cm under ground max temperature
    h_st030n: FLOAT,      // degC, 30 cm under ground min temperature
    h_st030nt: NaiveTime, // local time, time at 30 cm under ground min temperature
    h_st050: FLOAT,       // degC, 50 cm under ground temperature
    h_st050x: FLOAT,      // degC, 50 cm under ground max temperature
    h_st050xt: NaiveTime, // local time, time at 50 cm under ground max temperature
    h_st050n: FLOAT,      // degC, 50 cm under ground min temperature
    h_st050nt: NaiveTime, // local time, time at 50 cm under ground min temperature
    h_st100: FLOAT,       // degC, 100 cm under ground temperature
    h_st100x: FLOAT,      // degC, 100 cm under ground max temperature
    h_st100xt: NaiveTime, // local time, time at 100 cm under ground max temperature
    h_st100n: FLOAT,      // degC, 100 cm under ground min temperature
    h_st100nt: NaiveTime, // local time, time at 100 cm under ground min temperature
}

impl CWBHourData {
    pub fn parse_from_str(data: &str) -> Result<Self, Box<dyn Error + 'static>> {
        let mut words = data.split(',');
        if words.nth(2) != Some("HR") && words.nth(2) != Some("HH") {
            return Err(String::from("Invalid").into());
        }

        let mut words = data.split(',');

        // stx
        if words.next() != Some("\u{2}") {
            return Err(String::from("Invalid").into());
        }

        let result = CWBHourData {
            station: parse_string(&mut words)?,
            dkind: parse_string(&mut words)?,
            dtime: parse_datetime(&mut words)?,
            h_p1: parse_float(&mut words)?,
            h_p2: parse_float(&mut words)?,
            h_a: parse_integer(&mut words)?,
            h_pp: parse_float(&mut words)?,
            h_e: parse_float(&mut words)?,
            h_e_: parse_float(&mut words)?,
            h_t: parse_float(&mut words)?,
            h_tx: parse_float(&mut words)?,
            h_tn: parse_float(&mut words)?,
            h_td: parse_float(&mut words)?,
            h_rh: parse_float(&mut words)?,
            h_dd: parse_integer(&mut words)?,
            h_ff: parse_float(&mut words)?,
            h_f10: parse_float(&mut words)?,
            h_10d: parse_integer(&mut words)?,
            h_f10t: parse_time(&mut words)?,
            h_fx: parse_float(&mut words)?,
            h_xd: parse_integer(&mut words)?,
            h_fxt: parse_time(&mut words)?,
            h_xd3: parse_integer(&mut words)?,
            h_fx3: parse_float(&mut words)?,
            h_r: parse_float(&mut words)?,
            h_gmt: parse_float(&mut words)?,
            h_3r: parse_float(&mut words)?,
            h_6r: parse_float(&mut words)?,
            h_9r: parse_float(&mut words)?,
            h_12r: parse_float(&mut words)?,
            h_24r: parse_float(&mut words)?,
            h_24rrr: parse_float(&mut words)?,
            h_gr: parse_float(&mut words)?,
            h_sh: parse_float(&mut words)?,
            h_t005: parse_float(&mut words)?,
            h_t005x: parse_float(&mut words)?,
            h_t005xt: parse_time(&mut words)?,
            h_t005n: parse_float(&mut words)?,
            h_t005nt: parse_time(&mut words)?,
            h_t000: parse_float(&mut words)?,
            h_t000x: parse_float(&mut words)?,
            h_t000xt: parse_time(&mut words)?,
            h_t000n: parse_float(&mut words)?,
            h_t000nt: parse_time(&mut words)?,
            h_st005: parse_float(&mut words)?,
            h_st005x: parse_float(&mut words)?,
            h_st005xt: parse_time(&mut words)?,
            h_st005n: parse_float(&mut words)?,
            h_st005nt: parse_time(&mut words)?,
            h_st010: parse_float(&mut words)?,
            h_st010x: parse_float(&mut words)?,
            h_st010xt: parse_time(&mut words)?,
            h_st010n: parse_float(&mut words)?,
            h_st010nt: parse_time(&mut words)?,
            h_st020: parse_float(&mut words)?,
            h_st020x: parse_float(&mut words)?,
            h_st020xt: parse_time(&mut words)?,
            h_st020n: parse_float(&mut words)?,
            h_st020nt: parse_time(&mut words)?,
            h_st030: parse_float(&mut words)?,
            h_st030x: parse_float(&mut words)?,
            h_st030xt: parse_time(&mut words)?,
            h_st030n: parse_float(&mut words)?,
            h_st030nt: parse_time(&mut words)?,
            h_st050: parse_float(&mut words)?,
            h_st050x: parse_float(&mut words)?,
            h_st050xt: parse_time(&mut words)?,
            h_st050n: parse_float(&mut words)?,
            h_st050nt: parse_time(&mut words)?,
            h_st100: parse_float(&mut words)?,
            h_st100x: parse_float(&mut words)?,
            h_st100xt: parse_time(&mut words)?,
            h_st100n: parse_float(&mut words)?,
            h_st100nt: parse_time(&mut words)?,
        };

        // etx
        if words.next() != Some("\u{3}") {
            return Err(String::from("Invalid").into());
        }

        Ok(result)
    }
}

#[derive(Debug, Default)]
pub struct CWBDayData {
    station: String,
    dkind: String,
    dtime: NaiveDateTime, // local time
    d_mp1: FLOAT,         // hPa, station pressure
    d_dp1: FLOAT,         // hPa, pressure diff
    d_p1x: FLOAT,         // hPa, max station pressure
    d_p1xt: NaiveTime,    // local time, time at max station pressure
    d_p1n: FLOAT,         // hPa, min station pressure
    d_p1nt: NaiveTime,    // local time, time at min station pressure
    d_mp2: FLOAT,         // hPa, sea level pressure
    d_mt: FLOAT,          // degC, temperature
    d_dt: FLOAT,          // degC, temperature diff
    d_tx: FLOAT,          // degC, max temperature
    d_txt: NaiveTime,     // local time, time at max temperature
    d_tn: FLOAT,          // degC, min temperature
    d_tnt: NaiveTime,     // local time, time at min temperature
    d_mtd: FLOAT,         // degC, dew point temperature
    d_dtd: FLOAT,         // degC, dew point temperature diff
    d_tdx: FLOAT,         // degC, max dew point temperature
    d_tdxt: NaiveTime,    // local time, time at max dew point temperature
    d_tdn: FLOAT,         // degC, min dew point temperature
    d_tdnt: NaiveTime,    // local time, time at min dew point temperature
    d_mrh: FLOAT,         // %, relative humidity
    d_rhx: FLOAT,         // %, max relative humidity
    d_rhxt: NaiveTime,    // local time, time at max relative humidity
    d_rhn: FLOAT,         // %, min relative humidity
    d_rhnt: NaiveTime,    // local time, time at min relative humidity
    d_me_: FLOAT,         // hPa, vapor
    d_ex: FLOAT,          // hPa, max staturated vapor
    d_ext: NaiveTime,     // local time, time at max staturated vapor
    d_en: FLOAT,          // hPa, min staturated vapor
    d_ent: NaiveTime,     // local time, time at min staturated vapor
    d_me: FLOAT,          // hPa, staturated vapor
    d_mwd: INTEGER,       // vector, average wind direction
    d_mws: FLOAT,         // m/s, average wind speed
    d_fx: FLOAT,          // m/s, max wind speed
    d_xd: INTEGER,        // vector, wind direction at max wind speed
    d_fxt: NaiveTime,     // local time, time at max wind speed
    d_f10: FLOAT,         // m/s, max average wind speed
    d_10d: INTEGER,       // vector, wind direction at max average wind speed
    d_f10t: NaiveTime,    // local time, time at max average wind speed
    d_wd: FLOAT,          // m, wind range
    d_tr: FLOAT,          // mm, rain
    d_rx10: FLOAT,        // mm, max 10 min rain
    d_x10t: NaiveTime,    // local time, time at max 10 min rain
    d_rx60: FLOAT,        // mm, max 60 min rain
    d_x60t: NaiveTime,    // local time, time at max 60 min rain
    d_tgr: FLOAT,         // MJ/m2, accumulation of solar radiaiton
    d_tsh: FLOAT,         // Hr, accumulation of sunshine
    d_t005: FLOAT,        // degC, 5 cm grassland temperature
    d_t005x: FLOAT,       // degC, 5 cm grassland max temperature
    d_t005xt: NaiveTime,  // local time, time at 5 cm grassland max temperature
    d_t005n: FLOAT,       // degC, 5 cm grassland min temperature
    d_t005nt: NaiveTime,  // local time, time at 5 cm grassland min temperature
    d_t000: FLOAT,        // degC, 0 cm grassland temperature
    d_t000x: FLOAT,       // degC, 0 cm grassland max temperature
    d_t000xt: NaiveTime,  // local time, time at 0 cm grassland max temperature
    d_t000n: FLOAT,       // degC, 0 cm grassland min temperature
    d_t000nt: NaiveTime,  // local time, time at 0 cm grassland min temperature
    d_st005: FLOAT,       // degC, 5 cm under ground temperature
    d_st005x: FLOAT,      // degC, 5 cm under ground max temperature
    d_st005xt: NaiveTime, // local time, time at 5 cm under ground max temperature
    d_st005n: FLOAT,      // degC, 5 cm under ground min temperature
    d_st005nt: NaiveTime, // local time, time at 5 cm under ground min temperature
    d_st010: FLOAT,       // degC, 10 cm under ground temperature
    d_st010x: FLOAT,      // degC, 10 cm under ground max temperature
    d_st010xt: NaiveTime, // local time, time at 10 cm under ground max temperature
    d_st010n: FLOAT,      // degC, 10 cm under ground min temperature
    d_st010nt: NaiveTime, // local time, time at 10 cm under ground min temperature
    d_st020: FLOAT,       // degC, 20 cm under ground temperature
    d_st020x: FLOAT,      // degC, 20 cm under ground max temperature
    d_st020xt: NaiveTime, // local time, time at 20 cm under ground max temperature
    d_st020n: FLOAT,      // degC, 20 cm under ground min temperature
    d_st020nt: NaiveTime, // local time, time at 20 cm under ground min temperature
    d_st030: FLOAT,       // degC, 30 cm under ground temperature
    d_st030x: FLOAT,      // degC, 30 cm under ground max temperature
    d_st030xt: NaiveTime, // local time, time at 30 cm under ground max temperature
    d_st030n: FLOAT,      // degC, 30 cm under ground min temperature
    d_st030nt: NaiveTime, // local time, time at 30 cm under ground min temperature
    d_st050: FLOAT,       // degC, 50 cm under ground temperature
    d_st050x: FLOAT,      // degC, 50 cm under ground max temperature
    d_st050xt: NaiveTime, // local time, time at 50 cm under ground max temperature
    d_st050n: FLOAT,      // degC, 50 cm under ground min temperature
    d_st050nt: NaiveTime, // local time, time at 50 cm under ground min temperature
    d_st100: FLOAT,       // degC, 100 cm under ground temperature
    d_st100x: FLOAT,      // degC, 100 cm under ground max temperature
    d_st100xt: NaiveTime, // local time, time at 100 cm under ground max temperature
    d_st100n: FLOAT,      // degC, 100 cm under ground min temperature
    d_st100nt: NaiveTime, // local time, time at 100 cm under ground min temperature
}

impl CWBDayData {
    pub fn parse_from_str(data: &str) -> Result<Self, Box<dyn Error + 'static>> {
        let mut words = data.split(',');
        if words.nth(2) != Some("HR") && words.nth(2) != Some("HH") {
            return Err(String::from("Invalid").into());
        }

        let mut words = data.split(',');

        // stx
        if words.next() != Some("\u{2}") {
            return Err(String::from("Invalid").into());
        }

        let result = CWBDayData {
            station: parse_string(&mut words)?,
            dkind: parse_string(&mut words)?,
            dtime: parse_datetime(&mut words)?,
            d_mp1: parse_float(&mut words)?,
            d_dp1: parse_float(&mut words)?,
            d_p1x: parse_float(&mut words)?,
            d_p1xt: parse_time(&mut words)?,
            d_p1n: parse_float(&mut words)?,
            d_p1nt: parse_time(&mut words)?,
            d_mp2: parse_float(&mut words)?,
            d_mt: parse_float(&mut words)?,
            d_dt: parse_float(&mut words)?,
            d_tx: parse_float(&mut words)?,
            d_txt: parse_time(&mut words)?,
            d_tn: parse_float(&mut words)?,
            d_tnt: parse_time(&mut words)?,
            d_mtd: parse_float(&mut words)?,
            d_dtd: parse_float(&mut words)?,
            d_tdx: parse_float(&mut words)?,
            d_tdxt: parse_time(&mut words)?,
            d_tdn: parse_float(&mut words)?,
            d_tdnt: parse_time(&mut words)?,
            d_mrh: parse_float(&mut words)?,
            d_rhx: parse_float(&mut words)?,
            d_rhxt: parse_time(&mut words)?,
            d_rhn: parse_float(&mut words)?,
            d_rhnt: parse_time(&mut words)?,
            d_me_: parse_float(&mut words)?,
            d_ex: parse_float(&mut words)?,
            d_ext: parse_time(&mut words)?,
            d_en: parse_float(&mut words)?,
            d_ent: parse_time(&mut words)?,
            d_me: parse_float(&mut words)?,
            d_mwd: parse_integer(&mut words)?,
            d_mws: parse_float(&mut words)?,
            d_fx: parse_float(&mut words)?,
            d_xd: parse_integer(&mut words)?,
            d_fxt: parse_time(&mut words)?,
            d_f10: parse_float(&mut words)?,
            d_10d: parse_integer(&mut words)?,
            d_f10t: parse_time(&mut words)?,
            d_wd: parse_float(&mut words)?,
            d_tr: parse_float(&mut words)?,
            d_rx10: parse_float(&mut words)?,
            d_x10t: parse_time(&mut words)?,
            d_rx60: parse_float(&mut words)?,
            d_x60t: parse_time(&mut words)?,
            d_tgr: parse_float(&mut words)?,
            d_tsh: parse_float(&mut words)?,
            d_t005: parse_float(&mut words)?,
            d_t005x: parse_float(&mut words)?,
            d_t005xt: parse_time(&mut words)?,
            d_t005n: parse_float(&mut words)?,
            d_t005nt: parse_time(&mut words)?,
            d_t000: parse_float(&mut words)?,
            d_t000x: parse_float(&mut words)?,
            d_t000xt: parse_time(&mut words)?,
            d_t000n: parse_float(&mut words)?,
            d_t000nt: parse_time(&mut words)?,
            d_st005: parse_float(&mut words)?,
            d_st005x: parse_float(&mut words)?,
            d_st005xt: parse_time(&mut words)?,
            d_st005n: parse_float(&mut words)?,
            d_st005nt: parse_time(&mut words)?,
            d_st010: parse_float(&mut words)?,
            d_st010x: parse_float(&mut words)?,
            d_st010xt: parse_time(&mut words)?,
            d_st010n: parse_float(&mut words)?,
            d_st010nt: parse_time(&mut words)?,
            d_st020: parse_float(&mut words)?,
            d_st020x: parse_float(&mut words)?,
            d_st020xt: parse_time(&mut words)?,
            d_st020n: parse_float(&mut words)?,
            d_st020nt: parse_time(&mut words)?,
            d_st030: parse_float(&mut words)?,
            d_st030x: parse_float(&mut words)?,
            d_st030xt: parse_time(&mut words)?,
            d_st030n: parse_float(&mut words)?,
            d_st030nt: parse_time(&mut words)?,
            d_st050: parse_float(&mut words)?,
            d_st050x: parse_float(&mut words)?,
            d_st050xt: parse_time(&mut words)?,
            d_st050n: parse_float(&mut words)?,
            d_st050nt: parse_time(&mut words)?,
            d_st100: parse_float(&mut words)?,
            d_st100x: parse_float(&mut words)?,
            d_st100xt: parse_time(&mut words)?,
            d_st100n: parse_float(&mut words)?,
            d_st100nt: parse_time(&mut words)?,
        };

        // etx
        if words.next() != Some("\u{3}") {
            return Err(String::from("Invalid").into());
        }

        Ok(result)
    }
}

#[derive(Debug, Default)]
pub struct CWBSoilMinData {
    stationid: String,
    dkind: String,
    dtime: NaiveDateTime, // local time
    dd_vmc010: FLOAT,     // %,   0-10  cm soil water contain
    dd_vmc020: FLOAT,     // %,  10-20  cm soil water contain
    dd_vmc030: FLOAT,     // %,  20-30  cm soil water contain
    dd_vmc040: FLOAT,     // %,  30-40  cm soil water contain
    dd_vmc050: FLOAT,     // %,  40-50  cm soil water contain
    dd_vmc060: FLOAT,     // %,  50-60  cm soil water contain
    dd_vmc070: FLOAT,     // %,  60-70  cm soil water contain
    dd_vmc080: FLOAT,     // %,  70-80  cm soil water contain
    dd_vmc090: FLOAT,     // %,  80-90  cm soil water contain
    dd_vmc100: FLOAT,     // %,  90-100 cm soil water contain
    dd_vmc110: FLOAT,     // %, 100-110 cm soil water contain
    dd_vmc120: FLOAT,     // %, 110-120 cm soil water contain
    dd_sitest: String,    //  station status code
}

impl CWBSoilMinData {
    pub fn parse_from_str(data: &str) -> Result<Self, Box<dyn Error + 'static>> {
        let mut words = data.split(',');
        if words.nth(2) != Some("SM") {
            return Err(String::from("Invalid").into());
        }

        let mut words = data.split(',');

        // stx
        if words.next() != Some("\u{2}") {
            return Err(String::from("Invalid").into());
        }

        let result = CWBSoilMinData {
            stationid: parse_string(&mut words)?,
            dkind: parse_string(&mut words)?,
            dtime: parse_datetime(&mut words)?,
            dd_vmc010: parse_float(&mut words)?,
            dd_vmc020: parse_float(&mut words)?,
            dd_vmc030: parse_float(&mut words)?,
            dd_vmc040: parse_float(&mut words)?,
            dd_vmc050: parse_float(&mut words)?,
            dd_vmc060: parse_float(&mut words)?,
            dd_vmc070: parse_float(&mut words)?,
            dd_vmc080: parse_float(&mut words)?,
            dd_vmc090: parse_float(&mut words)?,
            dd_vmc100: parse_float(&mut words)?,
            dd_vmc110: parse_float(&mut words)?,
            dd_vmc120: parse_float(&mut words)?,
            dd_sitest: parse_string(&mut words)?,
        };

        // etx
        if words.next() != Some("\u{3}") {
            return Err(String::from("Invalid").into());
        }

        Ok(result)
    }
}

#[derive(Debug, Default)]
pub struct CWBSoilHourData {
    stationid: String,
    dkind: String,
    dtime: NaiveDateTime,  // local time
    h_vmc010: FLOAT,       // %,   0-10  cm soil water contain
    h_vmc010x: FLOAT,      // %,   0-10  cm max soil water contain
    h_vmc010xt: NaiveTime, // local time, time at 0-10 cm max soil water contain
    h_vmc010n: FLOAT,      // %,   0-10  cm min soil water contain
    h_vmc010nt: NaiveTime, // local time, time at 0-10 cm min soil water contain
    h_vmc020: FLOAT,       // %,  10-20  cm soil water contain
    h_vmc020x: FLOAT,      // %,  10-20  cm max soil water contain
    h_vmc020xt: NaiveTime, // local time, time at 10-20 cm max soil water contain
    h_vmc020n: FLOAT,      // %,  10-20  cm min soil water contain
    h_vmc020nt: NaiveTime, // local time, time at 10-20 cm min soil water contain
    h_vmc030: FLOAT,       // %,  20-30  cm soil water contain
    h_vmc030x: FLOAT,      // %,  20-30  cm max soil water contain
    h_vmc030xt: NaiveTime, // local time, time at 20-30 cm max soil water contain
    h_vmc030n: FLOAT,      // %,  20-30  cm min soil water contain
    h_vmc030nt: NaiveTime, // local time, time at 20-30 cm min soil water contain
    h_vmc040: FLOAT,       // %,  30-40  cm soil water contain
    h_vmc040x: FLOAT,      // %,  30-40  cm max soil water contain
    h_vmc040xt: NaiveTime, // local time, time at 30-40 cm max soil water contain
    h_vmc040n: FLOAT,      // %,  30-40  cm min soil water contain
    h_vmc040nt: NaiveTime, // local time, time at 30-40 cm min soil water contain
    h_vmc050: FLOAT,       // %,  40-50  cm soil water contain
    h_vmc050x: FLOAT,      // %,  40-50  cm max soil water contain
    h_vmc050xt: NaiveTime, // local time, time at 40-50 cm max soil water contain
    h_vmc050n: FLOAT,      // %,  40-50  cm min soil water contain
    h_vmc050nt: NaiveTime, // local time, time at 40-50 cm min soil water contain
    h_vmc060: FLOAT,       // %,  50-60  cm soil water contain
    h_vmc060x: FLOAT,      // %,  50-60  cm max soil water contain
    h_vmc060xt: NaiveTime, // local time, time at 50-60 cm max soil water contain
    h_vmc060n: FLOAT,      // %,  50-60  cm min soil water contain
    h_vmc060nt: NaiveTime, // local time, time at 50-60 cm min soil water contain
    h_vmc070: FLOAT,       // %,  60-70  cm soil water contain
    h_vmc070x: FLOAT,      // %,  60-70  cm max soil water contain
    h_vmc070xt: NaiveTime, // local time, time at 60-70 cm max soil water contain
    h_vmc070n: FLOAT,      // %,  60-70  cm min soil water contain
    h_vmc070nt: NaiveTime, // local time, time at 60-70 cm min soil water contain
    h_vmc080: FLOAT,       // %,  70-80  cm soil water contain
    h_vmc080x: FLOAT,      // %,  70-80  cm max soil water contain
    h_vmc080xt: NaiveTime, // local time, time at 70-80 cm max soil water contain
    h_vmc080n: FLOAT,      // %,  70-80  cm min soil water contain
    h_vmc080nt: NaiveTime, // local time, time at 70-80 cm min soil water contain
    h_vmc090: FLOAT,       // %,  80-90  cm soil water contain
    h_vmc090x: FLOAT,      // %,  80-90  cm max soil water contain
    h_vmc090xt: NaiveTime, // local time, time at 80-90 cm max soil water contain
    h_vmc090n: FLOAT,      // %,  80-90  cm min soil water contain
    h_vmc090nt: NaiveTime, // local time, time at 80-90 cm min soil water contain
    h_vmc100: FLOAT,       // %,  90-100 cm soil water contain
    h_vmc100x: FLOAT,      // %,  90-100 cm max soil water contain
    h_vmc100xt: NaiveTime, // local time, time at 90-100 cm max soil water contain
    h_vmc100n: FLOAT,      // %,  90-100 cm min soil water contain
    h_vmc100nt: NaiveTime, // local time, time at 90-100 cm min soil water contain
    h_vmc110: FLOAT,       // %, 100-110 cm soil water contain
    h_vmc110x: FLOAT,      // %, 100-110  cm max soil water contain
    h_vmc110xt: NaiveTime, // local time, time at 100-110 cm max soil water contain
    h_vmc110n: FLOAT,      // %, 100-110  cm min soil water contain
    h_vmc110nt: NaiveTime, // local time, time at 100-110 cm min soil water contain
    h_vmc120: FLOAT,       // %, 110-120 cm soil water contain
    h_vmc120x: FLOAT,      // %, 110-120 cm max soil water contain
    h_vmc120xt: NaiveTime, // local time, time at 110-120 cm max soil water contain
    h_vmc120n: FLOAT,      // %, 110-120 cm min soil water contain
    h_vmc120nt: NaiveTime, // local time, time at 110-120 cm min soil water contain
}

impl CWBSoilHourData {
    pub fn parse_from_str(data: &str) -> Result<Self, Box<dyn Error + 'static>> {
        let mut words = data.split(',');
        if words.nth(2) != Some("SH") {
            return Err(String::from("Invalid").into());
        }

        let mut words = data.split(',');

        // stx
        if words.next() != Some("\u{2}") {
            return Err(String::from("Invalid").into());
        }

        let result = CWBSoilHourData {
            stationid: parse_string(&mut words)?,
            dkind: parse_string(&mut words)?,
            dtime: parse_datetime(&mut words)?,
            h_vmc010: parse_float(&mut words)?,
            h_vmc010x: parse_float(&mut words)?,
            h_vmc010xt: parse_time(&mut words)?,
            h_vmc010n: parse_float(&mut words)?,
            h_vmc010nt: parse_time(&mut words)?,
            h_vmc020: parse_float(&mut words)?,
            h_vmc020x: parse_float(&mut words)?,
            h_vmc020xt: parse_time(&mut words)?,
            h_vmc020n: parse_float(&mut words)?,
            h_vmc020nt: parse_time(&mut words)?,
            h_vmc030: parse_float(&mut words)?,
            h_vmc030x: parse_float(&mut words)?,
            h_vmc030xt: parse_time(&mut words)?,
            h_vmc030n: parse_float(&mut words)?,
            h_vmc030nt: parse_time(&mut words)?,
            h_vmc040: parse_float(&mut words)?,
            h_vmc040x: parse_float(&mut words)?,
            h_vmc040xt: parse_time(&mut words)?,
            h_vmc040n: parse_float(&mut words)?,
            h_vmc040nt: parse_time(&mut words)?,
            h_vmc050: parse_float(&mut words)?,
            h_vmc050x: parse_float(&mut words)?,
            h_vmc050xt: parse_time(&mut words)?,
            h_vmc050n: parse_float(&mut words)?,
            h_vmc050nt: parse_time(&mut words)?,
            h_vmc060: parse_float(&mut words)?,
            h_vmc060x: parse_float(&mut words)?,
            h_vmc060xt: parse_time(&mut words)?,
            h_vmc060n: parse_float(&mut words)?,
            h_vmc060nt: parse_time(&mut words)?,
            h_vmc070: parse_float(&mut words)?,
            h_vmc070x: parse_float(&mut words)?,
            h_vmc070xt: parse_time(&mut words)?,
            h_vmc070n: parse_float(&mut words)?,
            h_vmc070nt: parse_time(&mut words)?,
            h_vmc080: parse_float(&mut words)?,
            h_vmc080x: parse_float(&mut words)?,
            h_vmc080xt: parse_time(&mut words)?,
            h_vmc080n: parse_float(&mut words)?,
            h_vmc080nt: parse_time(&mut words)?,
            h_vmc090: parse_float(&mut words)?,
            h_vmc090x: parse_float(&mut words)?,
            h_vmc090xt: parse_time(&mut words)?,
            h_vmc090n: parse_float(&mut words)?,
            h_vmc090nt: parse_time(&mut words)?,
            h_vmc100: parse_float(&mut words)?,
            h_vmc100x: parse_float(&mut words)?,
            h_vmc100xt: parse_time(&mut words)?,
            h_vmc100n: parse_float(&mut words)?,
            h_vmc100nt: parse_time(&mut words)?,
            h_vmc110: parse_float(&mut words)?,
            h_vmc110x: parse_float(&mut words)?,
            h_vmc110xt: parse_time(&mut words)?,
            h_vmc110n: parse_float(&mut words)?,
            h_vmc110nt: parse_time(&mut words)?,
            h_vmc120: parse_float(&mut words)?,
            h_vmc120x: parse_float(&mut words)?,
            h_vmc120xt: parse_time(&mut words)?,
            h_vmc120n: parse_float(&mut words)?,
            h_vmc120nt: parse_time(&mut words)?,
        };

        // etx
        if words.next() != Some("\u{3}") {
            return Err(String::from("Invalid").into());
        }

        Ok(result)
    }
}

#[derive(Debug, Default)]
pub struct CWBSoilDayData {
    stationid: String,
    dkind: String,
    dtime: NaiveDateTime,  // local time
    d_vmc010: FLOAT,       // %,   0-10  cm soil water contain
    d_vmc010x: FLOAT,      // %,   0-10  cm max soil water contain
    d_vmc010xt: NaiveTime, // local time, time at 0-10 cm max soil water contain
    d_vmc010n: FLOAT,      // %,   0-10  cm min soil water contain
    d_vmc010nt: NaiveTime, // local time, time at 0-10 cm min soil water contain
    d_vmc020: FLOAT,       // %,  10-20  cm soil water contain
    d_vmc020x: FLOAT,      // %,  10-20  cm max soil water contain
    d_vmc020xt: NaiveTime, // local time, time at 10-20 cm max soil water contain
    d_vmc020n: FLOAT,      // %,  10-20  cm min soil water contain
    d_vmc020nt: NaiveTime, // local time, time at 10-20 cm min soil water contain
    d_vmc030: FLOAT,       // %,  20-30  cm soil water contain
    d_vmc030x: FLOAT,      // %,  20-30  cm max soil water contain
    d_vmc030xt: NaiveTime, // local time, time at 20-30 cm max soil water contain
    d_vmc030n: FLOAT,      // %,  20-30  cm min soil water contain
    d_vmc030nt: NaiveTime, // local time, time at 20-30 cm min soil water contain
    d_vmc040: FLOAT,       // %,  30-40  cm soil water contain
    d_vmc040x: FLOAT,      // %,  30-40  cm max soil water contain
    d_vmc040xt: NaiveTime, // local time, time at 30-40 cm max soil water contain
    d_vmc040n: FLOAT,      // %,  30-40  cm min soil water contain
    d_vmc040nt: NaiveTime, // local time, time at 30-40 cm min soil water contain
    d_vmc050: FLOAT,       // %,  40-50  cm soil water contain
    d_vmc050x: FLOAT,      // %,  40-50  cm max soil water contain
    d_vmc050xt: NaiveTime, // local time, time at 40-50 cm max soil water contain
    d_vmc050n: FLOAT,      // %,  40-50  cm min soil water contain
    d_vmc050nt: NaiveTime, // local time, time at 40-50 cm min soil water contain
    d_vmc060: FLOAT,       // %,  50-60  cm soil water contain
    d_vmc060x: FLOAT,      // %,  50-60  cm max soil water contain
    d_vmc060xt: NaiveTime, // local time, time at 50-60 cm max soil water contain
    d_vmc060n: FLOAT,      // %,  50-60  cm min soil water contain
    d_vmc060nt: NaiveTime, // local time, time at 50-60 cm min soil water contain
    d_vmc070: FLOAT,       // %,  60-70  cm soil water contain
    d_vmc070x: FLOAT,      // %,  60-70  cm max soil water contain
    d_vmc070xt: NaiveTime, // local time, time at 60-70 cm max soil water contain
    d_vmc070n: FLOAT,      // %,  60-70  cm min soil water contain
    d_vmc070nt: NaiveTime, // local time, time at 60-70 cm min soil water contain
    d_vmc080: FLOAT,       // %,  70-80  cm soil water contain
    d_vmc080x: FLOAT,      // %,  70-80  cm max soil water contain
    d_vmc080xt: NaiveTime, // local time, time at 70-80 cm max soil water contain
    d_vmc080n: FLOAT,      // %,  70-80  cm min soil water contain
    d_vmc080nt: NaiveTime, // local time, time at 70-80 cm min soil water contain
    d_vmc090: FLOAT,       // %,  80-90  cm soil water contain
    d_vmc090x: FLOAT,      // %,  80-90  cm max soil water contain
    d_vmc090xt: NaiveTime, // local time, time at 80-90 cm max soil water contain
    d_vmc090n: FLOAT,      // %,  80-90  cm min soil water contain
    d_vmc090nt: NaiveTime, // local time, time at 80-90 cm min soil water contain
    d_vmc100: FLOAT,       // %,  90-100 cm soil water contain
    d_vmc100x: FLOAT,      // %,  90-100 cm max soil water contain
    d_vmc100xt: NaiveTime, // local time, time at 90-100 cm max soil water contain
    d_vmc100n: FLOAT,      // %,  90-100 cm min soil water contain
    d_vmc100nt: NaiveTime, // local time, time at 90-100 cm min soil water contain
    d_vmc110: FLOAT,       // %, 100-110 cm soil water contain
    d_vmc110x: FLOAT,      // %, 100-110  cm max soil water contain
    d_vmc110xt: NaiveTime, // local time, time at 100-110 cm max soil water contain
    d_vmc110n: FLOAT,      // %, 100-110  cm min soil water contain
    d_vmc110nt: NaiveTime, // local time, time at 100-110 cm min soil water contain
    d_vmc120: FLOAT,       // %, 110-120 cm soil water contain
    d_vmc120x: FLOAT,      // %, 110-120 cm max soil water contain
    d_vmc120xt: NaiveTime, // local time, time at 110-120 cm max soil water contain
    d_vmc120n: FLOAT,      // %, 110-120 cm min soil water contain
    d_vmc120nt: NaiveTime, // local time, time at 110-120 cm min soil water contain
}

impl CWBSoilDayData {
    pub fn parse_from_str(data: &str) -> Result<Self, Box<dyn Error + 'static>> {
        let mut words = data.split(',');
        if words.nth(2) != Some("SD") {
            return Err(String::from("Invalid").into());
        }

        let mut words = data.split(',');

        // stx
        if words.next() != Some("\u{2}") {
            return Err(String::from("Invalid").into());
        }

        let result = CWBSoilDayData {
            stationid: parse_string(&mut words)?,
            dkind: parse_string(&mut words)?,
            dtime: parse_datetime(&mut words)?,
            d_vmc010: parse_float(&mut words)?,
            d_vmc010x: parse_float(&mut words)?,
            d_vmc010xt: parse_time(&mut words)?,
            d_vmc010n: parse_float(&mut words)?,
            d_vmc010nt: parse_time(&mut words)?,
            d_vmc020: parse_float(&mut words)?,
            d_vmc020x: parse_float(&mut words)?,
            d_vmc020xt: parse_time(&mut words)?,
            d_vmc020n: parse_float(&mut words)?,
            d_vmc020nt: parse_time(&mut words)?,
            d_vmc030: parse_float(&mut words)?,
            d_vmc030x: parse_float(&mut words)?,
            d_vmc030xt: parse_time(&mut words)?,
            d_vmc030n: parse_float(&mut words)?,
            d_vmc030nt: parse_time(&mut words)?,
            d_vmc040: parse_float(&mut words)?,
            d_vmc040x: parse_float(&mut words)?,
            d_vmc040xt: parse_time(&mut words)?,
            d_vmc040n: parse_float(&mut words)?,
            d_vmc040nt: parse_time(&mut words)?,
            d_vmc050: parse_float(&mut words)?,
            d_vmc050x: parse_float(&mut words)?,
            d_vmc050xt: parse_time(&mut words)?,
            d_vmc050n: parse_float(&mut words)?,
            d_vmc050nt: parse_time(&mut words)?,
            d_vmc060: parse_float(&mut words)?,
            d_vmc060x: parse_float(&mut words)?,
            d_vmc060xt: parse_time(&mut words)?,
            d_vmc060n: parse_float(&mut words)?,
            d_vmc060nt: parse_time(&mut words)?,
            d_vmc070: parse_float(&mut words)?,
            d_vmc070x: parse_float(&mut words)?,
            d_vmc070xt: parse_time(&mut words)?,
            d_vmc070n: parse_float(&mut words)?,
            d_vmc070nt: parse_time(&mut words)?,
            d_vmc080: parse_float(&mut words)?,
            d_vmc080x: parse_float(&mut words)?,
            d_vmc080xt: parse_time(&mut words)?,
            d_vmc080n: parse_float(&mut words)?,
            d_vmc080nt: parse_time(&mut words)?,
            d_vmc090: parse_float(&mut words)?,
            d_vmc090x: parse_float(&mut words)?,
            d_vmc090xt: parse_time(&mut words)?,
            d_vmc090n: parse_float(&mut words)?,
            d_vmc090nt: parse_time(&mut words)?,
            d_vmc100: parse_float(&mut words)?,
            d_vmc100x: parse_float(&mut words)?,
            d_vmc100xt: parse_time(&mut words)?,
            d_vmc100n: parse_float(&mut words)?,
            d_vmc100nt: parse_time(&mut words)?,
            d_vmc110: parse_float(&mut words)?,
            d_vmc110x: parse_float(&mut words)?,
            d_vmc110xt: parse_time(&mut words)?,
            d_vmc110n: parse_float(&mut words)?,
            d_vmc110nt: parse_time(&mut words)?,
            d_vmc120: parse_float(&mut words)?,
            d_vmc120x: parse_float(&mut words)?,
            d_vmc120xt: parse_time(&mut words)?,
            d_vmc120n: parse_float(&mut words)?,
            d_vmc120nt: parse_time(&mut words)?,
        };

        // etx
        if words.next() != Some("\u{3}") {
            return Err(String::from("Invalid").into());
        }

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use chrono::NaiveDateTime;

    use super::*;
    #[test]
    fn case1() {
        let timestr = "2024-01-02 00:01:02";
        let time =
            NaiveDateTime::parse_from_str(timestr, "%Y-%m-%d %H:%M:%S").expect("System error");

        println!("{:?}", time);
    }
}
