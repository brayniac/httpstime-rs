// httpstime-rs - Rust httpstime client
//
// The MIT License (MIT)
//
// Copyright (c) 2015 Brian Martin
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#[macro_use]
extern crate log;
extern crate hyper;
extern crate time;
extern crate shuteye;
extern crate getopts;
extern crate curl;

use getopts::Options;
use hyper::client::Client;
use hyper::header::{Date, HttpDate, Headers, UserAgent};
use log::{Log, LogLevel, LogLevelFilter, LogMetadata, LogRecord};
use std::env;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("s", "server", "server", "HOST|HOST:PORT");
    opts.optopt("n", "num-polls", "number of times to poll", "INTEGER");
    opts.optflag("d", "debug", "debug logging");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => {
            m
        }
        Err(f) => {
            panic!(f.to_string())
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    if !matches.opt_present("s") {
        error!("require server parameter");
        print_usage(&program, opts);
        return;
    }
    let server = matches.opt_str("s").unwrap();

    let mut num_polls = 8;
    if matches.opt_present("n") {
        match matches.opt_str("n").unwrap().parse() {
            Ok(n) => {
                num_polls = n;
            }
            Err(_) => {
                panic!("Bad option for: {}", "-n|--num-polls");
            }
        }
    }

    let mut log_filter = LogLevelFilter::Info;

    if matches.opt_present("d") {
        log_filter = LogLevelFilter::Debug;
    }

    let _ = log::set_logger(|max_log_level| {
        max_log_level.set(log_filter);
        return Box::new(SimpleLogger);
    });

    debug!("httpstime-rs {} initializing...", VERSION);

    let hyper = Client::new();
    let mut client = curl::http::handle().ssl_verifypeer(true);

    let url = "https://".to_string() + &server + "/.well-known/time";
    let ua = format!("httpstime-rs/{}", VERSION);

    // make initial request to verify ssl
    match client.head(url.clone()).header("UserAgent", &ua).exec() {
        Ok(_) => {
            debug!("ssl verify OK");
        }
        Err(e) => {
            debug!("ssl verify FAIL! {}", e);
            println!("*RESULT 1 {} nan nan nan Verify fail: {}", server, e);
            return;
        }
    }

    let mut x0 = i64::min_value();
    let mut x1 = i64::max_value();

    for _ in 0..num_polls {
        let result = time_from_https(&hyper, url.clone(), ua.clone());
        let t0 = (result.t0 - time::Duration::seconds(1) - result.t2).num_milliseconds();
        let t1 = (result.t1 - result.t2).num_milliseconds();
        let rtt = (result.t1 - result.t0).num_milliseconds();

        if t0 > x0 {
            x0 = t0;

            if t1 < x1 {
                x1 = t1;
            }
            debug!("B {:?} {:?}", x0, x1);
        } else if t1 < x1 {
            x1 = t1;
            debug!("A {:?} {:?}", x0, x1);
        } else {
            debug!("C {:?} {:?}", x0, x1);
        }

        let mut dt = ((x1 + x0) / 2) - rtt / 2;

        while dt < 0 {
            dt = dt + 1000;
        }
        debug!("dt: {:?}", dt);

        let now = time::now_utc();

        let mut b = dt - (now.to_timespec().nsec as i64 / 1000000);

        while b < 0 {
            b = b + 1000;
        }
        while b > 1000 {
            b = b - 1000;
        }
        debug!("b: {:?}", b);

        shuteye::sleep(shuteye::Timespec::from_nano(b * 1000000).unwrap());
    }

    let low = (x0 as f64) / 1000_f64;
    let high = (x1 as f64) / 1000_f64;
    let width = high - low;

    println!("*RESULT 0 {} {:.*} {:.*} {:.*}",
             server,
             3,
             low,
             3,
             high,
             3,
             width);

}

struct HttpsTimes {
    t0: time::Timespec,
    t1: time::Timespec,
    t2: time::Timespec,
}

fn time_from_https(client: &Client, url: String, ua: String) -> HttpsTimes {
    let mut headers = Headers::new();
    headers.set(UserAgent(ua));

    debug!("HEAD {}", url);
    let t0 = time::now_utc().to_timespec();
    let res = client.head(&url).headers(headers).send().unwrap();
    let t1 = time::now_utc().to_timespec();

    let headers = res.headers.clone();
    let date = headers.get::<Date>().unwrap();

    debug!("Date: {}", date);

    let Date(HttpDate(tm)) = *date;

    debug!("Local: {:?}", tm);

    let tm_utc = tm.to_utc();

    debug!("UTC: {:?}", tm_utc);

    let t2 = tm_utc.to_timespec();

    debug!("t0: {:?}", t0);
    debug!("t1: {:?}", t1);
    debug!("t2: {:?}", t2);

    HttpsTimes {
        t0: t0,
        t1: t1,
        t2: t2,
    }
}

pub struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Debug
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            if record.location().module_path() == "httpstime_rs" {
                println!("{} {:<5} [{}] {}",
                         time::strftime("%Y-%m-%d %H:%M:%S", &time::now()).unwrap(),
                         record.level().to_string(),
                         "httpstime-rs",
                         record.args());
            }
        }
    }
}
