use crate::core::*;

use std::collections::HashMap;

use zen_colour::*;

pub struct SummaryData<'a>{
    pub namebank: &'a NameBank,
    pub state: &'a State,
    pub hist: &'a [Vec<f32>],
    pub redact: bool,
    pub redact_map: &'a HashMap<String, String>,
    pub includes: &'a [String],
    pub rounding: &'a str,
    pub min_asset_worth: f32,
}

pub fn summary(d: &SummaryData) -> f32{
    let accounts = into_named_accounts(&d.state.accounts, d.namebank, d.state);
    let amounts = into_named_assets(&d.state.asset_amounts, d.namebank);
    let prices = into_named_assets(&d.state.asset_prices, d.namebank);
    let it = amounts.iter().zip(prices.iter());
    let pos_sum: f32 = accounts.iter().skip(NR_BUILDIN_ACCOUNTS).map(|(_, x, stat)|
        if *x > 0.0 && !stat { *x } else { 0.0 }
    ).sum();
    let total_holdings_worth: f32 = it.fold(0.0, |acc, ((_, a), (_, p))| acc + a * p);
    let min_sum = pos_sum.min(total_holdings_worth);
    let norm_fac = if d.redact { min_sum } else { 1.0 };
    let net = accounts[NET].1;
    let debt = net - pos_sum;
    let r#yield = accounts[YIELD].1;
    // let roi = accounts[ROI].1;
    let assets = accounts[ASSETS].1;
    let sum_holding_error = pos_sum - total_holdings_worth;
    let fiat = amounts[0].1;
    let shadowrealm_fiat = amounts[1].1;
    let fiat_split = fiat / total_holdings_worth;
    let assets_split = 1.0 - fiat_split;
    let spend_past_12m: f32 = d.hist.iter().rev().take(12).map(|frame| frame[SPENDING_MONTH]).sum();
    let receive_past_12m: f32 = d.hist.iter().rev().take(12).map(|frame| frame[RECEIVING_MONTH]).sum();
    let saving_rate_past_12m = (receive_past_12m - spend_past_12m) / receive_past_12m * 100.0;
    let assets_pos_sum_error = assets - (pos_sum * assets_split);
    let assets_total_holdings_error = assets - (total_holdings_worth * assets_split);
    let assets_error = assets_pos_sum_error.max(assets_total_holdings_error);

    let (textc, infoc, namec, posc, negc, fracc, b, r) =
        (DEFAULT, MAGENTA, BLUE, GREEN, RED, YELLOW, BOLD, RESET);
    let pncol = |v: f32| if v < 0.0 { negc } else { posc };
    // let roicol = |v: f32| if v < 1.0 { negc } else { posc };

    let val = match d.rounding {
        "none" => |v: f32| v,
        "whole" => |v: f32| v.round(),
        _ => |v: f32| (v * 100.0).round() / 100.0,
    };

    println!("{}{b}General{r}:", infoc);
    println!("  {}Net: {}{}{}", textc, pncol(net), val(net / norm_fac), textc);
    println!("  {}Debt: {}{}{}", textc, pncol(debt), val(debt / norm_fac), textc);
    println!("  {}Yield: {}{}{}", textc, pncol(r#yield), val(r#yield / norm_fac), textc);
    println!("  {}Assets: {}{}{}", textc, pncol(assets), val(assets / norm_fac), textc);
    println!("  {}Fiat: {}{}{}", textc, pncol(fiat), val(fiat / norm_fac), textc);
    println!("  {}Positive owned sum: {}{}", textc, posc, val(if d.redact { 1.0 } else { pos_sum }));
    println!("  {}Total holdings worth: {}{}",
        textc, posc, val(total_holdings_worth / norm_fac)
    );
    println!("  {}Positive owned sum / holdings error: {}{}{} which is {}{}{}%",
        textc, pncol(sum_holding_error), sum_holding_error / norm_fac, textc,
        posc, sum_holding_error.abs() / min_sum * 100.0, textc
    );
    println!("  {}Assets / (positive sum, holdings) error: {}{}{} which is {}{}{}%",
        textc, pncol(assets_error), assets_error / norm_fac, textc, posc,
        assets_error.abs() / min_sum * 100.0, textc
    );
    println!("  {}You spent {}{}{} the past year",
        textc, pncol(spend_past_12m), val(spend_past_12m / norm_fac), textc
    );
    println!("  {}You received {}{}{} the past year",
        textc, pncol(receive_past_12m), val(receive_past_12m / norm_fac), textc
    );
    println!("  {}Your saving rate is {}{}{}% the past year",
        textc, pncol(saving_rate_past_12m), val(saving_rate_past_12m), textc
    );

    println!("{b}{}Accounts{r}:", infoc);
    let mut to_print = Vec::new();
    let include_not_everything = !d.includes.is_empty();
    for (name, amount, _) in &accounts{
        let index = d.includes.iter().position(|inc| inc == name);
        if include_not_everything && index.is_none(){ continue; }
        let val = *amount / norm_fac;
        let name = if let Some(redacted) = d.redact_map.get(name){
            redacted
        } else {
            name
        };
        to_print.push((name, val, index.unwrap_or(0)));
    }
    if include_not_everything{
        to_print.sort_by(|(_, _, i), (_, _, j)| i.cmp(j));
    }
    for (name, aval, _) in to_print{
        println!("  {}{}: {}{}", namec, name, pncol(aval), val(aval));
    }

    println!("{b}{}Distribution{r}:", infoc);
    println!("  {t}With a split of {f}{a}{t}% assets and {f}{b}{t}% fiat",
        t = textc, f = fracc, a = val(assets_split * 100.0), b = val(fiat_split * 100.0)
    );
    println!("  {t}A total of {c}{f}{t} fiat is stuck in the shadowrealm",
        t = textc, c = pncol(shadowrealm_fiat), f = shadowrealm_fiat / norm_fac
    );

    let mut data_rows = Vec::new();
    for ((name, amount), (_, price)) in amounts.iter().zip(prices.iter()){
        if *price == 0.0 { continue; }
        if *amount < 0.000001 { continue; }
        let worth = amount * price;
        if worth < d.min_asset_worth { continue; }
        data_rows.push((name, amount, worth, price, worth / total_holdings_worth));
    }
    data_rows.sort_by(|(_, _, _, _, sa), (_, _, _, _, sb)|
        sb.partial_cmp(sa).unwrap_or(std::cmp::Ordering::Less)
    );
    for (name, amount, worth, price, share) in data_rows{
        if !d.redact{
            println!("  {nc}{name}{tc}: {ac}{amount}{tc} worth {wc}{worth}{tc} priced {pc}{price}{tc} at {sc}{share}{tc}% of total",
                tc = textc, nc = namec, name = name, ac = pncol(*amount), amount = val(*amount),
                wc = pncol(worth), worth = val(worth), pc = pncol(*price), price = val(*price),
                sc = fracc, share = val(share * 100.0)
            );
        } else {
            println!("  {nc}{name}{tc} at {sc}{share}{tc}% of total",
                tc = textc, nc = namec, name = name, sc = fracc, share = val(share * 100.0));
        }
    }

    println!("{b}{}Metrics{r}:", infoc);
    let time_flat = net / spend_past_12m * 12.0;
    let moy = |x: f32| if x.abs() > 24.0 { x / 12.0 } else { x }; // months or years
    let moy_label = |x: f32| if x.abs() > 24.0 { "years" } else { "months" };
    println!("  {}Your net worth is {}{}{} {} (no Inflation and ROI)",
        textc, pncol(time_flat), val(moy(time_flat)), textc, moy_label(time_flat));
    println!("  {}A {}2{}% yield would give you {}{}{}% of your spending.",
        textc, posc, textc, posc, val((min_sum * 0.02) / spend_past_12m * 100.0), textc);

    let print_time_exp = |inflation_rate: f32, roi_rate: f32|{
        let inflation = (1.0 + (inflation_rate * 0.01)).powf(1.0 / 12.0);
        let roi = (1.0 + (roi_rate * 0.01)).powf(1.0 / 12.0);
        let infc = if inflation > 1.0 { negc } else { posc };
        let roic = if roi > 1.0 { posc } else { negc };
        let mut month_cost = spend_past_12m / 12.0;
        let mut assets = min_sum - fiat;
        let mut total = min_sum;
        let mut months = 0.0;
        loop{
            if months >= 1200.0{
                println!("  {}Your assets are worth {}100+{} years ({}% Infl., {}% ROI)",
                    textc, posc, textc, inflation_rate, roi_rate
                );
                return;
            }
            if total > month_cost{
                total -= month_cost;
                month_cost *= inflation;
                months += 1.0;
                assets = assets.min(total);
                total -= assets;
                assets *= roi;
                total += assets;
            } else {
                months += total / month_cost;
                println!("  {}Your assets are worth {}{}{} {} ({}{}{}% Infl., {}{}{}% ROI)",
                    textc, pncol(months), val(moy(months)), textc, moy_label(months), infc,
                    inflation_rate, textc, roic, roi_rate, textc
                );
                return;
            }
        }
    };

    print_time_exp(10.0, -10.0);
    print_time_exp(10.0, -5.0);
    print_time_exp(10.0, 0.0);
    print_time_exp(5.0, -5.0);
    print_time_exp(5.0, 0.0);
    print_time_exp(5.0, 5.0);
    print_time_exp(5.0, 6.0);
    print_time_exp(5.0, 7.0);
    print_time_exp(5.0, 9.0);
    norm_fac
}
