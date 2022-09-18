use web3::{
    ethabi::ethereum_types::U256,
};

pub fn to_hex_str(volume: &str) -> String{
    let vol = volume.to_string().parse::<f64>().unwrap();
    let vals = ((1000000000000000000.0*vol) as u64).to_string();
    let val_u = U256::from_dec_str(&vals).unwrap();
    return format!("0x{:x}", val_u);
}

pub fn to_hex_unprepared_str(volume: &str) -> String{
    let vol = volume.to_string().parse::<f64>().unwrap();
    let vals = (vol as u64).to_string();
    let val_u = U256::from_dec_str(&vals).unwrap();
    return format!("0x{:x}", val_u);
}

pub fn to_hex_str_clean(volume: &str) -> String{
    let vol = volume.to_string().parse::<f64>().unwrap();
    let vals = ((1000000000000000000.0*vol) as u64).to_string();
    let val_u = U256::from_dec_str(&vals).unwrap();
    return format!("{:x}", val_u);
}

pub fn to_hex_str_prepared_clean(volume: &str) -> String{
    let val_u = U256::from_dec_str(volume).unwrap();
    return format!("{:x}", val_u);
}

pub fn to_U256(volume: &str) -> U256{
    let vol = volume.to_string().parse::<f64>().unwrap();
    let vals = ((1000000000000000000.0*vol) as u64).to_string();
    let val_u = U256::from_dec_str(&vals).unwrap();
    return val_u;
}

pub fn hxt_str_to_f64(volume: &str) -> f64{
    let res1 : &str = &volume[2..volume.len()].trim_start_matches('0');
    if (res1.len()==0){
        return 0.0;
    } else{
        let res_dec = i128::from_str_radix(&res1, 16).unwrap();
        let res_f = (res_dec as f64)/1000000000000000000.0;
        return res_f;
    }
}
