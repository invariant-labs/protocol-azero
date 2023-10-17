use ink::storage::Mapping;
use openbrush::traits::{AccountId, Balance};

use crate::contract::{OrderPair, TokenPairs};

use crate::contracts::pair::pair::PairField;

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct Pairs {
    pub pairs: Mapping<(AccountId, AccountId), PairField>,
    pub token_pairs: TokenPairs,
    pub length: u64,
}

impl Pairs {
    pub fn _create_pair(&mut self, ordered_pair: OrderPair) -> PairField {
        assert_ne!(ordered_pair.x.0, ordered_pair.y.0);
        let pair = self.pairs.get((&ordered_pair.x.0, &ordered_pair.y.0));
        match pair {
            Some(pair) => pair,
            None => {
                let pair_field = PairField {
                    token_x: ordered_pair.x.0,
                    token_y: ordered_pair.y.0,
                    ..Default::default()
                };
                self.pairs
                    .insert((ordered_pair.x.0, ordered_pair.y.0), &pair_field);
                self.length += 1;
                self.token_pairs
                    .0
                    .push((ordered_pair.x.0, ordered_pair.y.0));
                pair_field
            }
        }
    }
    pub fn _update_pair(&mut self, pair: PairField) {
        self.pairs.insert((pair.token_x, pair.token_y), &pair);
    }

    pub fn _get_pair(&self, ordered_pair: &OrderPair) -> Option<PairField> {
        self.pairs.get((ordered_pair.x.0, ordered_pair.y.0))
    }

    pub fn _order_tokens(
        &self,
        token_0: AccountId,
        token_1: AccountId,
        balance_0: Balance,
        balance_1: Balance,
    ) -> OrderPair {
        match token_0.lt(&token_1) {
            true => OrderPair {
                x: (token_0, balance_0),
                y: (token_1, balance_1),
            },
            false => OrderPair {
                x: (token_1, balance_1),
                y: (token_0, balance_0),
            },
        }
    }
}
