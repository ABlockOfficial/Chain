use crate::primitives::transaction::OutPoint;
use crate::utils::{add_btreemap, format_for_display};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt, iter, mem::size_of, ops};

/// A structure representing the amount of tokens in an instance
#[derive(Deserialize, Serialize, Default, Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct TokenAmount(pub u64);

impl fmt::Display for TokenAmount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let format_result = format_for_display(&self.0);
        write!(f, "{format_result}")
    }
}

impl ops::Add for TokenAmount {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match self.0.checked_add(other.0) {
            Some(v) => Self(v),
            None => Self(u64::MAX),
        }
    }
}

impl ops::AddAssign for TokenAmount {
    fn add_assign(&mut self, other: Self) {
        self.0 = match self.0.checked_add(other.0) {
            Some(v) => v,
            None => u64::MAX,
        }
    }
}

impl ops::Sub for TokenAmount {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        match self.0.checked_sub(other.0) {
            Some(v) => Self(v),
            None => Self(u64::MIN),
        }
    }
}

impl ops::SubAssign for TokenAmount {
    fn sub_assign(&mut self, other: Self) {
        self.0 = match self.0.checked_sub(other.0) {
            Some(v) => v,
            None => u64::MIN,
        }
    }
}

impl ops::Mul<u64> for TokenAmount {
    type Output = Self;

    fn mul(self, rhs: u64) -> Self {
        match self.0.checked_mul(rhs) {
            Some(v) => Self(v),
            None => Self(u64::MAX),
        }
    }
}

impl ops::MulAssign<u64> for TokenAmount {
    fn mul_assign(&mut self, rhs: u64) {
        self.0 = match self.0.checked_mul(rhs) {
            Some(v) => v,
            None => u64::MAX,
        }
    }
}

impl ops::Div<u64> for TokenAmount {
    type Output = Self;

    fn div(self, rhs: u64) -> Self {
        match self.0.checked_div(rhs) {
            Some(v) => Self(v),
            None => Self(u64::MAX),
        }
    }
}

impl ops::DivAssign<u64> for TokenAmount {
    fn div_assign(&mut self, rhs: u64) {
        self.0 = match self.0.checked_div(rhs) {
            Some(v) => v,
            None => u64::MAX,
        }
    }
}

impl iter::Sum for TokenAmount {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), |r, l| r + l)
    }
}

/// Item asset struct
#[derive(Default, Deserialize, Serialize, Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct ItemAsset {
    pub amount: u64,
    pub drs_tx_hash: Option<String>,
    pub metadata: Option<String>,
}

impl ItemAsset {
    pub fn new(amount: u64, drs_tx_hash: Option<String>, metadata: Option<String>) -> Self {
        Self {
            amount,
            drs_tx_hash,
            metadata,
        }
    }
}

/// Data asset struct
#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct DataAsset {
    pub data: Vec<u8>,
    pub amount: u64,
}

/// Asset struct
///
/// * `Token`   - An asset struct representation of the ZNT token
/// * `Data`    - A data asset
/// * `Item` - A item for a payment. The value indicates the number of item assets
#[derive(Deserialize, Serialize, Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum Asset {
    Token(TokenAmount),
    Data(DataAsset),
    Item(ItemAsset),
}

impl Default for Asset {
    fn default() -> Self {
        Asset::Token(Default::default())
    }
}

impl Asset {
    /// Modify `self` of `Asset` struct to obtain `drs_tx_hash`
    /// from either the asset itself or its corresponding `OutPoint`
    pub fn with_fixed_hash(mut self, out_point: &OutPoint) -> Self {
        if let Asset::Item(ref mut item_asset) = self {
            if item_asset.drs_tx_hash.is_none() {
                item_asset.drs_tx_hash = Some(&out_point.t_hash).cloned();
            }
        }
        self
    }

    /// Get optional `drs_tx_hash` value for `Asset`
    pub fn get_drs_tx_hash(&self) -> Option<&String> {
        match self {
            Asset::Token(_) => None,
            Asset::Data(_) => None, /* TODO: This will have to change */
            Asset::Item(item) => item.drs_tx_hash.as_ref(),
        }
    }

    pub fn get_metadata(&self) -> Option<&String> {
        match self {
            Asset::Token(_) => None,
            Asset::Data(_) => None,
            Asset::Item(item) => item.metadata.as_ref(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Asset::Token(_) => size_of::<TokenAmount>(),
            Asset::Data(d) => d.data.len(),
            Asset::Item(_) => size_of::<u64>(),
        }
    }

    pub fn token_u64(amount: u64) -> Self {
        Asset::Token(TokenAmount(amount))
    }

    pub fn item(amount: u64, drs_tx_hash: Option<String>, metadata: Option<String>) -> Self {
        Asset::Item(ItemAsset::new(amount, drs_tx_hash, metadata))
    }

    /// Add an asset of the same variant to `self` asset.
    /// TODO: Add handling for `Data` asset variant. Will return false when `Data` asset is presented.
    ///
    /// ### Note
    ///
    /// This function will return false for `Item` assets
    /// getting added together that do not have the same `drs_tx_hash`
    ///
    /// ### Arguments
    ///
    ///* `rhs`          - The right-hand-side (RHS) asset to add to `self`
    pub fn add_assign(&mut self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Asset::Token(lhs_tokens), Asset::Token(rhs_tokens)) => {
                *lhs_tokens += *rhs_tokens;
                true
            }
            (Asset::Item(lhs_items), Asset::Item(rhs_items)) => {
                if lhs_items.drs_tx_hash != rhs_items.drs_tx_hash {
                    return false;
                }
                lhs_items.amount += rhs_items.amount;
                true
            }
            _ => false,
        }
    }

    /// Determine if `self` asset is greater or equal to another asset of the same variant.
    /// TODO: Add handling for `Data` asset variant. Will return None if `Data` asset is presented.
    ///
    /// ### Arguments
    ///
    ///* `rhs`                  - Reference to right-hand-side (RHS) `Asset`
    pub fn is_greater_or_equal_to(&self, rhs: &Asset) -> Option<bool> {
        match (&self, &rhs) {
            (Asset::Token(lhs_token_amount), Asset::Token(rhs_token_amount)) => {
                Some(lhs_token_amount >= rhs_token_amount)
            }
            (Asset::Item(lhs_item), Asset::Item(rhs_item)) => {
                if lhs_item.drs_tx_hash != rhs_item.drs_tx_hash {
                    return None;
                }
                Some(lhs_item.amount >= rhs_item.amount)
            }
            _ => None,
        }
    }

    /// Determine if `self` asset is greater than another asset of the same variant.
    /// If `self` asset is greater, return the excess.
    /// TODO: Add handling for `Data` asset variant. Will return None if `Data` asset is presented.
    ///
    /// ### Arguments
    ///
    ///* `rhs`                  - Reference to right-hand-side (RHS) `Asset`
    pub fn get_excess(&self, rhs: &Asset) -> Option<Asset> {
        match (&self, &rhs) {
            (Asset::Token(lhs_tokens), Asset::Token(rhs_tokens)) => {
                if lhs_tokens > rhs_tokens {
                    Some(Asset::Token(*lhs_tokens - *rhs_tokens))
                } else {
                    None
                }
            }
            (Asset::Item(lhs_items), Asset::Item(rhs_items)) => {
                if lhs_items.amount > rhs_items.amount
                    && lhs_items.drs_tx_hash == rhs_items.drs_tx_hash
                {
                    Some(Asset::item(
                        lhs_items.amount - rhs_items.amount,
                        lhs_items.drs_tx_hash.clone(),
                        lhs_items.metadata.clone(),
                    ))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Determine if the asset in question is of the same variant as `self`
    ///
    /// ### Arguments
    ///
    ///* `other`  - Reference to other `Asset` to test against
    pub fn is_same_type_as(&self, other: &Asset) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }

    /// Creates a default asset of a given variant.
    /// TODO: Add handling for `Data` asset variant
    ///
    /// ### Arguments
    ///
    ///* `asset_type`          - Default asset variant type
    pub fn default_of_type(asset_type: &Self) -> Self {
        match asset_type {
            Self::Token(_) => Self::Token(Default::default()),
            Self::Item(item) => Self::item(
                Default::default(),
                item.drs_tx_hash.clone(),
                item.metadata.clone(),
            ),
            _ => panic!("Cannot create default of asset type: {:?}", asset_type),
        }
    }

    pub fn is_token(&self) -> bool {
        matches!(self, Asset::Token(_))
    }

    pub fn is_item(&self) -> bool {
        matches!(self, Asset::Item(_))
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Asset::Data(d) => d.data.is_empty(),
            _ => false,
        }
    }

    pub fn token_amount(&self) -> TokenAmount {
        match self {
            Asset::Token(v) => *v,
            _ => TokenAmount(0),
        }
    }

    pub fn item_amount(&self) -> u64 {
        match self {
            Asset::Item(v) => v.amount,
            _ => 0,
        }
    }
}

/// `AssetValue` struct used to represent the a running total of `Token` and `Item` assets
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetValues {
    pub tokens: TokenAmount,
    // Note: Items from create transactions will have `drs_tx_hash` = `t_hash`
    pub items: BTreeMap<String, u64>, /* `drs_tx_hash` - amount */
}

impl ops::AddAssign for AssetValues {
    fn add_assign(&mut self, rhs: Self) {
        self.tokens += rhs.tokens;
        add_btreemap(&mut self.items, rhs.items);
    }
}

impl AssetValues {
    pub fn new(tokens: TokenAmount, items: BTreeMap<String, u64>) -> Self {
        Self { tokens, items }
    }

    pub fn token_u64(tokens: u64) -> Self {
        AssetValues::new(TokenAmount(tokens), Default::default())
    }

    pub fn item(items: BTreeMap<String, u64>) -> Self {
        AssetValues::new(TokenAmount(0), items)
    }

    pub fn is_empty(&self) -> bool {
        self == &AssetValues::default()
    }

    pub fn is_equal(&self, rhs: &AssetValues) -> bool {
        self.tokens == rhs.tokens && self.items == rhs.items
    }

    // See if the running total is enough for a required `Asset` amount
    pub fn has_enough(&self, asset_required: &Asset) -> bool {
        match asset_required {
            Asset::Token(tokens) => self.tokens >= *tokens,
            Asset::Item(items) => {
                if let Some(drs_tx_hash) = &items.drs_tx_hash {
                    self.items
                        .get(drs_tx_hash)
                        .map_or(false, |amount| *amount >= items.amount)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Add the `rhs` parameter to `self`
    pub fn update_add(&mut self, rhs: &Asset) {
        match rhs {
            Asset::Token(tokens) => self.tokens += *tokens,
            Asset::Item(items) => {
                if let Some(drs_tx_hash) = &items.drs_tx_hash {
                    self.items
                        .entry(drs_tx_hash.clone())
                        .and_modify(|amount| *amount += items.amount)
                        .or_insert(items.amount);
                }
            }
            _ => {}
        }
    }

    // Subtract the `rhs` parameter from `self`
    pub fn update_sub(&mut self, rhs: &Asset) {
        match rhs {
            Asset::Token(tokens) => self.tokens -= *tokens,
            Asset::Item(items) => {
                items.drs_tx_hash.as_ref().and_then(|drs_tx_hash| {
                    self.items
                        .get_mut(drs_tx_hash)
                        .map(|amount| *amount -= items.amount)
                });
            }
            _ => {}
        }
    }
}

#[test]
fn test_token_amount_operations() {
    // add
    let token1: TokenAmount = TokenAmount(u64::MAX - 1);
    let token2: TokenAmount = TokenAmount(2);
    assert_eq!(token1 + token2, TokenAmount(u64::MAX));
    // add_assign
    let mut token1: TokenAmount = TokenAmount(u64::MAX - 1);
    let token2: TokenAmount = TokenAmount(2);
    token1 += token2;
    assert_eq!(token1, TokenAmount(u64::MAX));
    // sub
    let token1: TokenAmount = TokenAmount(u64::MIN);
    let token2: TokenAmount = TokenAmount(1);
    assert_eq!(token1 - token2, TokenAmount(u64::MIN));
    // sub_assign
    let mut token1: TokenAmount = TokenAmount(u64::MIN);
    let token2: TokenAmount = TokenAmount(1);
    token1 -= token2;
    assert_eq!(token1, TokenAmount(u64::MIN));
    // mul
    let token: TokenAmount = TokenAmount(u64::MAX - 1);
    let rhs: u64 = 2;
    assert_eq!(token * rhs, TokenAmount(u64::MAX));
    // mul_assign
    let mut token: TokenAmount = TokenAmount(u64::MAX - 1);
    let rhs: u64 = 2;
    token *= rhs;
    assert_eq!(token, TokenAmount(u64::MAX));
    // div
    let token: TokenAmount = TokenAmount(1);
    let rhs: u64 = 0;
    assert_eq!(token / rhs, TokenAmount(u64::MAX));
    // div_assign
    let mut token: TokenAmount = TokenAmount(1);
    let rhs: u64 = 0;
    token /= rhs;
    assert_eq!(token, TokenAmount(u64::MAX));
}
