use super::json_scheme::{ActionItem, KyokuMeta, RawLog, ResultItem};
use crate::{KyokuFilter, Tile};

use serde::Serialize;
use serde_json::{self as json, Value};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("invalid json: {source}")]
    InvalidJSON {
        #[from]
        source: json::Error,
    },
    #[error("not three-player game")]
    NotThreePlayer,
    #[error("invalid hora detail")]
    InvalidHoraDetail,
}

/// The overview structure of log in tenhou.net/6 format.
#[derive(Debug, Clone)]
pub struct Log {
    pub names: [String; 4],
    pub game_length: GameLength,
    pub has_aka: bool,
    pub kyokus: Vec<Kyoku>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum GameLength {
    Hanchan = 0,
    Tonpuu = 4,
}

/// Contains information about a kyoku.
#[derive(Debug, Clone)]
pub struct Kyoku {
    pub meta: KyokuMeta,
    pub scoreboard: [i32; 4],
    pub dora_indicators: Vec<Tile>,
    pub ura_indicators: Vec<Tile>,
    pub action_tables: [ActionTable; 4],
    pub end_status: EndStatus,
}

#[derive(Debug, Clone)]
pub enum EndStatus {
    Hora { details: Vec<HoraDetail> },
    Ryukyoku { score_deltas: [i32; 4] },
}

#[derive(Debug, Clone, Default)]
pub struct HoraDetail {
    pub who: u8,
    pub target: u8,
    pub score_deltas: [i32; 4],
}

/// A group of "配牌", "取" and "出", describing a player's
/// gaming status and actions throughout a kyoku.
#[derive(Debug, Clone)]
pub struct ActionTable {
    pub haipai: Vec<Tile>,
    pub takes: Vec<ActionItem>,
    pub discards: Vec<ActionItem>,
}

impl Log {
    /// Parse a tenhou.net/6 log from JSON string.
    #[inline]
    pub fn from_json_str(json_string: &str) -> Result<Self, ParseError> {
        let raw_log: RawLog = json::from_str(json_string)?;
        Self::try_from(raw_log)
    }

    #[inline]
    pub fn filter_kyokus(&mut self, kyoku_filter: &KyokuFilter) {
        self.kyokus
            .retain(|l| kyoku_filter.test(l.meta.kyoku_num, l.meta.honba));
    }
}

impl TryFrom<RawLog> for Log {
    type Error = ParseError;

    fn try_from(raw_log: RawLog) -> Result<Self, Self::Error> {
        let RawLog {
            logs, names, rule, ..
        } = raw_log;

        if rule.disp.contains('四') || rule.disp.contains("4-Player") {
            return Err(ParseError::NotThreePlayer);
        }
        let game_length = if rule.disp.contains('東') || rule.disp.contains("East") {
            GameLength::Tonpuu
        } else {
            GameLength::Hanchan
        };
        let has_aka = rule.aka + rule.aka51 + rule.aka52 + rule.aka53 > 0;

        let mut kyokus = Vec::with_capacity(logs.len());
        for log in logs {
            let mut kyoku = Kyoku {
                meta: log.meta,
                scoreboard: log.scoreboard,
                dora_indicators: log.dora_indicators,
                ura_indicators: log.ura_indicators,
                action_tables: [
                    ActionTable {
                        haipai: log.haipai_0,
                        takes: log.takes_0,
                        discards: log.discards_0,
                    },
                    ActionTable {
                        haipai: log.haipai_1,
                        takes: log.takes_1,
                        discards: log.discards_1,
                    },
                    ActionTable {
                        haipai: log.haipai_2,
                        takes: log.takes_2,
                        discards: log.discards_2,
                    },
                    ActionTable {
                        haipai: log.haipai_3,
                        takes: log.takes_3,
                        discards: log.discards_3,
                    },
                ],
                end_status: EndStatus::Ryukyoku {
                    score_deltas: [0; 4], // default
                },
            };

            if let Some(ResultItem::Status(status_text)) = log.results.first() {
                if status_text == "和了" {
                    let mut details = vec![];
                    for detail_tuple in log.results[1..].chunks_exact(2) {
                        if let [ResultItem::ScoreDeltas(score_deltas), ResultItem::HoraDetail(who_target_tuple)] =
                            detail_tuple
                        {
                            let who = if let Some(Value::Number(n)) = who_target_tuple.first() {
                                n.as_u64().unwrap_or(0) as u8
                            } else {
                                return Err(ParseError::InvalidHoraDetail);
                            };
                            let target = if let Some(Value::Number(n)) = who_target_tuple.get(1) {
                                n.as_u64().unwrap_or(0) as u8
                            } else {
                                return Err(ParseError::InvalidHoraDetail);
                            };
                            let hora_detail = HoraDetail {
                                score_deltas: *score_deltas,
                                who,
                                target,
                            };
                            details.push(hora_detail);
                        }
                    }
                    kyoku.end_status = EndStatus::Hora { details };
                } else {
                    let score_deltas =
                        if let Some(ResultItem::ScoreDeltas(dts)) = log.results.get(1) {
                            *dts
                        } else {
                            [0; 4]
                        };
                    kyoku.end_status = EndStatus::Ryukyoku { score_deltas };
                }
            }

            kyokus.push(kyoku);
        }

        Ok(Self {
            names,
            game_length,
            has_aka,
            kyokus,
        })
    }
}

#[cfg(test)]

mod test
{
    use super::*;

    #[test]
    fn test_parse_log() {
        let json_str = r#"{"ver":2.3,"ref":"2024030511gm-00b9-0000-e0c07689","log":[[[0,0,0],[35000,35000,35000,0],[47],[],[26,27,32,33,35,37,37,39,41,42,44,44,46],[45,47,19,39,27,34,21,43,26],[60,60,60,60,60,60,"f44","f44",42],[11,19,21,23,29,31,33,41,42,44,44,45,46],[42,28,22,19,"4242p42",23,22,24],["f44",33,"f44",60,19,31,11,41],[19,21,29,29,29,31,34,34,38,41,43,47,47],[21,"47p4747",42,35,27,26,"34p3434",25],[19,43,60,31,41,38,35],[],[],[],["和了",[-700,-400,1100,0],[2,2,2,"40符1飜400-700点","役牌 中(1飜)"]]],[[1,0,0],[34300,34600,36100,0],[41],[],[19,24,27,29,31,31,31,32,34,35,39,39,46],[26,22,19,21,28,35,21,39,28],[19,46,60,29,32,21,60,24,39],[23,26,27,32,33,33,37,41,43,44,44,46,47],[23,24,36,46,"p464646",37,41,23,42,32,39,33],["f44","f44",41,43,47,24,60,36,60,32,60,32],[11,21,24,24,25,29,32,33,34,35,36,45,47],[25,38,25,47,27,22,22,37,41,28],[11,21,29,38,60,45,24,"r24",60,60],[],[],[],["和了",[0,8700,-7700,0],[1,2,1,"40符3飜7700点","役牌 發(1飜)","ドラ(2飜)"]]],[[1,1,0],[34300,43300,27400,0],[26],[],[11,25,27,31,33,38,39,41,42,43,43,45,46],[53,11,43,43,42,29,44,37,52,"42p4242",24],[11,60,45,41,43,46,29,"f44",27,31,60],[19,21,23,25,26,28,29,32,33,36,41,45,47],[25,36,27,28,26,21,23,34,46,22],[47,45,19,41,28,60,29,28,60,23],[11,24,28,31,33,34,35,36,37,38,38,42,44],[44,32,38,47,34,39,39,23,24,45,42,47],["f44","f44",11,60,42,28,24,60,60,60,60,60],[],[],[],["和了",[-3100,3100,0,0],[1,0,1,"30符2飜2900点","平和(1飜)","ドラ(1飜)"]]],[[1,2,0],[31200,46400,27400,0],[33],[],[25,31,33,35,36,41,41,44,45,47,47,47,47],[33,44,46,29,"33p3333",44,22,19,"41p4141",42,53,11,24,34,19,29,37,34],["f44","f44",25,60,46,"f44",60,60,47,60,31,60,60,35,60,60,45],[11,21,22,27,27,28,28,31,38,39,41,44,45],[38,26,42,42,45,36,32,43,26,36,11,46,23,27,46,11],["f44",11,60,60,31,60,60,41,22,21,60,60,60,60,60,60],[19,19,21,25,26,28,29,33,38,38,39,39,46],[27,36,28,41,23,37,35,37,23,34,32,42,39,22,43],[21,46,33,36,29,41,25,23,60,28,60,60,19,60,19],[],[],[],["和了",[12400,-8200,-4200,0],[0,0,0,"倍満4000-8000点","役牌 中(1飜)","場風 東(1飜)","混一色(2飜)","ドラ(5飜)","赤ドラ(1飜)"]]],[[2,0,0],[43600,38200,23200,0],[53],[47],[19,21,23,25,52,26,29,35,37,38,41,43,47],[39,34,27,44,28,42,32,45,28,19,42,21],[47,19,41,"f44",43,60,60,29,23,60,27,60],[23,24,31,32,33,36,36,38,41,43,44,45,46],[39,11,21,37,45,31,43,19,"3636p36",27,36,47,34],["f44",60,46,41,21,60,24,60,23,60,60,60,31],[22,24,25,26,28,29,33,33,35,38,38,41,47],[31,46,26,25,37,27,34,36,27,29,11,22,32],[47,60,41,22,29,28,"r37",60,60,60,60,60],[],[],[],["和了",[-1000,-1000,3000,0],[2,2,2,"30符2飜1000点∀","立直(1飜)","門前清自摸和(1飜)"]]],[[2,1,0],[42600,37200,25200,0],[29],[19],[24,26,29,32,33,35,36,37,43,43,44,46,47],[19,37,38,29,11,53,38,43,32,52,47,47,22,38,41],["f44",19,46,47,60,29,29,26,24,60,60,60,60,33,60],[21,22,23,24,25,28,33,34,34,39,42,44,46],[37,42,37,26,36,26,22,31,33,24,34,45,35,27,32],["f44",46,34,28,39,60,60,60,60,60,60,60,"r37",60],[21,22,23,26,31,31,33,35,36,38,39,45,46],[41,28,24,36,27,27,11,39,21,23,28,29,28,31],[45,46,41,39,24,60,60,60,60,60,60,60,60,33],[],[],[],["和了",[-2100,7200,-4100,0],[1,1,1,"満貫2000-4000点","立直(1飜)","門前清自摸和(1飜)","平和(1飜)","ドラ(2飜)"]]],[[4,0,0],[40500,43400,21100,0],[33],[],[11,19,26,29,31,31,32,32,33,35,43,46,47],[37,34,"3131p31",21,24,"32p3232",24,19,26,33,38,43],[43,11,29,60,60,26,60,46,60,19,19,60],[24,25,52,31,34,37,38,39,41,41,43,44,44],[46,11,39,36,21,19,28,22,37,29,53,36,33,39],["f44","f44",43,31,46,60,21,11,28,60,22,39,39,60],[22,23,25,27,28,32,34,36,36,38,42,42,46],[27,26,45,44,21,23,11,28,46,42,27,24],[46,38,60,"f44",32,34,60,36,36,46,23],[],[],[],["和了",[-6000,-3000,9000,0],[2,2,2,"跳満3000-6000点","門前清自摸和(1飜)","場風 南(1飜)","混一色(3飜)","ドラ(1飜)"]]],[[5,0,0],[34500,40400,30100,0],[47],[37],[11,23,24,52,26,32,32,33,36,38,42,44,45],[44,35,38,33,32,43,28,29,27,19,36,53,26],["f44","f44",11,42,45,60,60,60,33,60,60,33,36],[19,22,23,24,26,27,31,36,39,43,43,45,46],[23,25,31,37,35,44,39,46,28,37,31,28,25],[39,31,60,46,19,"f44",45,60,"r39",60,60,60,60],[19,21,22,23,25,27,29,29,29,32,33,34,37],[47,34,19,46,41,21,41,46,28,34,42,26],[60,19,60,60,60,37,60,60,34,60,60,"r21"],[],[],[],["和了",[0,6800,-5800,0],[1,2,1,"30符3飜5800点","立直(1飜)","平和(1飜)","ドラ(1飜)"]]],[[5,1,0],[34500,46200,24300,0],[34],[28],[23,24,25,26,31,32,32,34,38,39,42,44,46],[28,31,24,47,42,44,36,33,26,21,38,31,24,29],["f44",42,46,60,60,"f44",39,28,60,60,32,"r36",60,60],[21,23,33,34,37,38,38,41,41,43,44,45,47],[32,25,33,43,23,45,44,26,29,43,41,19,27],["f44",43,47,60,45,60,"f44",33,60,60,21,41,41],[11,19,19,21,22,27,33,36,37,41,42,42,47],[28,"42p4242",36,28,37,35,"28p2828",46,45,52,39,29,22],[11,41,47,33,22,21,27,60,60,60,60,60,60],[],[],[],["和了",[6400,0,-5400,0],[0,2,0,"40符3飜5200点","立直(1飜)","ドラ(2飜)"]]],[[6,0,0],[39900,46200,18900,0],[41,36],[47,36],[19,25,27,29,33,34,34,36,37,41,45,45,46],[19,22,37,28],[41,46,36,45],[23,24,24,27,31,31,33,37,39,41,42,43,43],[53,19,43,22],[41,42,19,53],[11,11,26,27,28,29,31,32,33,39,39,44,45],[35,47,11,38,27,11,52],["f44",45,47,60,"r35","111111a11"],[],[],[],["和了",[-4000,-4000,9000,0],[2,2,2,"満貫4000点∀","立直(1飜)","嶺上開花(1飜)","門前清自摸和(1飜)","ドラ(1飜)","赤ドラ(1飜)"]]],[[6,1,0],[35900,42200,26900,0],[25],[],[11,19,25,52,26,26,28,32,35,37,38,38,44],[24,33,46,37,47,39,37,31],["f44",19,11,46,28,35,47,38],[21,21,22,26,27,28,28,31,33,33,41,42,47],[32,41,47,27,45,"p474747",24],[41,60,42,33,60,22,60],[11,22,23,24,26,28,29,29,34,38,43,44,46],[41,44,19,36,23,25,23,19,31],["f44","f44",11,19,41,46,43,60,60],[],[],[],["和了",[12200,-12200,0,0],[0,1,0,"跳満12000点","平和(1飜)","一盃口(1飜)","ドラ(3飜)","赤ドラ(1飜)"]]]],"connection":[{"what":0,"log":0,"who":0,"step":3},{"what":1,"log":0,"who":0,"step":40}],"ratingc":"PF3","rule":{"disp":"三鳳南喰赤","aka53":1,"aka52":1,"aka51":1},"lobby":0,"dan":["七段","天鳳","八段","新人"],"rate":[2221.9,2461.48,2227.63,1500],"sx":["M","M","M","C"],"sc":[48100,43.1,30000,-10,26900,-33.1,0,0],"name":["mtk","つくねん3","ひぐお3",""]}
    "#;
        let log = Log::from_json_str(json_str).unwrap();
        assert_eq!(log.names[0], "mtk");
        assert_eq!(log.names[1], "つくねん3");
        assert_eq!(log.names[2], "ひぐお3");
        assert_eq!(log.names[3], "");
        assert_eq!(log.kyokus.len(), 11);
    }
}
