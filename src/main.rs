mod core;
mod summary;
mod graph;

use crate::core::*;
use crate::summary::*;
use crate::graph::*;

use std::fs;

fn main() {
    let args = lapp::parse_args("
        Tells you how poor you are.
        -r, --redact redact absolute valuations
        -g, --graph draw draw
        -p, --palette (default \'\') file to read colours from
        -c, --colours (integer...) lines to get colours from (bg, fg, col0, col1, ...)
        -b, --browser (default firefox) browser to show graph in
        --graph-accounts (string...) accounts to graph
        --summary-accounts (string...) accounts to include in the summary account listing
        --date-year-digits (default 4) how many digits to display a date's year with: [0,1,2,3,4]
        --date-month-digit use a digit instead of a 3 letter name for a date's month
        <file> (string) transactional \"database\" file
    ");
    let infile = args.get_string("file");
    let redact = args.get_bool("redact");
    let draw_graph = args.get_bool("graph");
    let contents = fs::read_to_string(infile).expect("Couldn't read sample.");
    let browser = args.get_string("browser");
    let year_digits = args.get_integer("date-year-digits").min(4).max(0) as u16;
    let use_month_name = !args.get_bool("date-month-digit");
    let mut namebank = NameBank::new();
    let mut date = Date::default();
    let ts = contents.split('\n').into_iter().map(|line| line.to_string()
        .into_trans(&mut namebank, &mut date)).flatten().collect::<Vec<_>>();
    let norm_fac = summary(&namebank, &ts, redact, &args.get_strings("summary-accounts"));

    if draw_graph{
        let colours = get_graph_colours(&args);
        let includes = args.get_strings("graph-accounts");
        if !includes.is_empty(){
            let includes = includes.iter().map(|s| s.as_str()).collect::<Vec<_>>();
            graph(norm_fac, &namebank, &ts, &includes, colours, &browser, year_digits, use_month_name);
        }
    }
}

