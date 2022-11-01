use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeConfig{
  pub nodeurl               : String
 ,pub nodeport              : u16
 ,pub dbname                : String
 ,pub dburl                 : String
 ,pub etherscan_api_key     : String
 ,pub etherscan_api_prefix  : String
 ,pub alchemy_api_key       : String
 ,pub api_call_timeout      : u64
}

impl NodeConfig {
    pub fn naddr(self) -> String {
        let adr = self.nodeurl + ":" + &self.nodeport.to_string();
        return adr;
    }
}


impl ::std::default::Default for NodeConfig {
    fn default() -> Self { Self { nodeurl: "http://localhost".into()
                                , nodeport:8545
                                , dbname:"accs.db".into()
                                , dburl:"postgres://hexstody:hexstody@localhost:5432/hexstody".into()
                                , etherscan_api_key:"P8AXZC7V71IJA4XPMFEIIYX9S2S4D8U3T6".into()
                                , etherscan_api_prefix:"api-ropsten".into()
                                , alchemy_api_key:"BZqEzKfIa6KJwWUYEum4ENKcqwMpPm7Z".into()
                                , api_call_timeout:30
                        }}
}


pub fn load_config(confname: &str) -> NodeConfig {
    let mcontent = std::fs::read_to_string(confname);
    let dcfg: NodeConfig = Default::default();
    match mcontent {
        Ok(cont) => {
            let mcfg = serde_json::from_str(&cont);
            match mcfg {
                Ok(cfg) => { return cfg;}
                Err(_)  => { println!("file parsing err, using default config"); return dcfg;}
            }
        }
        Err(_)   => { println!("file loading err, using default config"); return dcfg;}
    }
}
