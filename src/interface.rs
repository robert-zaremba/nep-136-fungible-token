use near_sdk::json_types::U128;
use near_sdk::AccountId;

struct Metadata {
    name: String,       // token name
    symbol: String,     // token symbol
    reference: String,  // URL to additional resources about the token.
    granularity: uint8, // the smallest part of the token that’s (denominated in e18) not divisible
    decimals: uint8,    // MUST be 18,
}

/// NEP
pub trait TransferCallRecipient {
    fn metadata() -> Metadata;

    /// Returns total supply.
    /// MUST equal to total_amount_of_token_minted - total_amount_of_token_burned
    fn total_supply(&self) -> U128;

    /// Returns the token balance for `holder` account
    fn balance_of(&self, holder: AccountId) -> U128;

    /// Transfer `amount` of tokens from the predecessor account to a `recipient` account.
    /// If recipient is a smart-contract, then `transfer_call` should be used instead.
    /// `recipient` MUST NOT be a smart-contract.
    /// `msg`: is a message for recipient. It might be used to send additional call
    //      instructions.
    /// `memo`: arbitrary data with no specified format used to link the transaction with an
    ///     external data. If referencing a binary data, it should use base64 serialization.
    ///
    /// ### Panics when
    /// - the predecessor doesn't have sufficient amount of shares.
    /// - if the predecessor is not registered (by calling `register_account`).
    #[payable]
    fn transfer(&mut self, recipient: AccountId, amount: U128, msg: String, memo: String) -> bool;

    /// Transfer `amount` of tokens from the predecessor account to a `recipient` contract.
    /// `recipient` MUST be a smart contract address.
    /// The recipient contract MUST implement `TransferCallRecipient` interface.
    /// `msg`: is a message sent to the recipient. It might be used to send additional call
    //      instructions.
    /// `memo`: arbitrary data with no specified format used to link the transaction with an
    ///     external event. If referencing a binary data, it should use base64 serialization.
    ///
    /// ### Panics when
    /// - the predecessor doesn't have sufficient amount of shares.
    /// - if the predecessor is not registered (by calling `register_account`).
    #[payable]
    fn transfer_call(
        &mut self,
        recipient: AccountId,
        amount: U128,
        msg: String,
        memo: String,
    ) -> bool;

    /// Registers the caller for accepting token transfers. Caller must deposit enough NEAR
    /// to cover storage cost.
    /// MUST not panic if caller is already registered.
    #[payable]
    fn register_account(&mut self);

    /// Checks if caller is registered (through `register_account`).
    fn is_account_registered(&mut self) -> bool;

    /// Unregisters the caller for accepting token transfers. Caller must deposit enough NEAR
    /// to cover storage cost.
    /// MUST panic if caller balance != 0.
    fn unregister_account(&mut self);

    /// OPTIONAL method
    /// burns `amount` of tokens from caller balance.
    ///
    /// ### Panics when
    /// - amount > balance_of()
    fn burn(&mut self, amount: U128);
}

/// Interface for recipient call on fungible-token transfers.
/// `token` is an account address of the token  - a smart-contract defining the token
///     being transferred.
/// `from` is an address of a previous holder of the tokens being sent
pub trait TransferCallRecipient {
    fn on_ft_receive(&mut self, token: AccountId, from: AccountId, amount: U128, msg: String);
}
