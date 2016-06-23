// pest. Elegant, efficient grammars
// Copyright (C) 2016  Dragoș Tiselice
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![recursion_limit = "80"]
#![feature(test)]

extern crate test;

#[macro_use]
extern crate pest;

use std::collections::{HashMap, LinkedList};
use std::fs::File;
use std::io::Read;

use test::Bencher;

use pest::prelude::*;

#[derive(Debug)]
pub enum JSONObject<'a> {
    Null,
    False,
    True,
    Number(f64),
    String(&'a str),
    Array(Vec<JSONObject<'a>>),
    Object(HashMap<String, JSONObject<'a>>)
}

impl_rdp! {
    grammar! {
        json = _{ value ~ eoi }

        object = { ["{"] ~ pair ~ ([","] ~ pair)* ~ ["}"] | ["{"] ~ ["}"] }
        pair   = { string ~ [":"] ~ value }

        array = { ["["] ~ value ~ ([","] ~ value)* ~ ["]"] | ["["] ~ ["]"] }

        value = { string | number | object | array | true_ | false_ | null }

        true_ = { ["true"] }
        false_ = { ["false"] }
        null = { ["null"] }

        string  = @{ ["\""] ~ content ~ ["\""] }
        content =  { (escape | !(["\""] | ["\\"]) ~ any)* }
        escape  = _{ ["\\"] ~ (["\""] | ["\\"] | ["/"] | ["b"] | ["f"] | ["n"] | ["r"] | ["t"] | unicode) }
        unicode = _{ ["u"] ~ hex ~ hex ~ hex ~ hex }
        hex     = _{ ['0'..'9'] | ['a'..'f'] | ['A'..'F'] }

        number = @{ ["-"]? ~ int ~ (["."] ~ ['0'..'9']+ ~ exp? | exp)? }
        int    = _{ ["0"] | ['1'..'9'] ~ ['0'..'9']* }
        exp    = _{ (["E"] | ["e"]) ~ (["+"] | ["-"])? ~ int }

        whitespace = _{ [" "] | ["\t"] | ["\r"] | ["\n"] }
    }

    process! {
        _json(&self) -> JSONObject<'input> {
            (_: value, value: _value()) => value
        }

        _value(&self) -> JSONObject<'input> {
            (_: string, &content: content) => {
                JSONObject::String(content)
            },
            (&number: number) => {
                JSONObject::Number(number.parse::<f64>().unwrap())
            },
            (_: true_) => {
                JSONObject::True
            },
            (_: false_) => {
                JSONObject::False
            },
            (_: null) => {
                JSONObject::Null
            },
            (_: array) => {
                let mut array = vec![];

                loop {
                    if let Some(token) = self.queue().get(self.queue_index()) {
                        if token.rule == Rule::value {
                            self.inc_queue_index();

                            array.push(self._value());
                        } else {
                            break
                        }
                    } else {
                        break
                    }
                }

                JSONObject::Array(array)
            },
            (_: object) => {
                let mut map = HashMap::new();

                loop {
                    if let Some(token) = self.queue().get(self.queue_index()) {
                        if token.rule == Rule::pair {
                            self.inc_queue_index();

                            let pair = self._pair();

                            map.insert(pair.0, pair.1);
                        } else {
                            break
                        }
                    } else {
                        break
                    }
                }

                JSONObject::Object(map)
            }
        }

        _pair(&self) -> (String, JSONObject<'input>) {
            (_: string, &key: content, _: value, value: _value()) => {
                (key.to_owned(), value)
            }
        }
    }
}

#[bench]
fn data(b: &mut Bencher) {
    let mut file = File::open("benches2/large.json").unwrap();
    let mut data = String::new();

    file.read_to_string(&mut data).unwrap();

    let mut parser = Rdp::new(StringInput::new(&data));

    b.iter(|| {
        parser.json();
        let a = parser._json();

        match a {
            JSONObject::Array(v) => {
                println!("{:?}", v[0]);
            },
            _ => ()
        }

        parser.reset();
    });
}
