# NEP Initeractive Fungible Token


We present a new fungible token standard for [NEAR Protocol](https://near.org) which is designed to be interactive (transfers can call other contracts) and simplified and friendly in asychronous environment like NEAR native runtime.

* &#x1F449; &nbsp;  **[Smart contract interface](./src/interface.rs)**.

## Context

This NEP is based on authors work on [NEARswap](https://github.com/near-clp/smart-contracts#our-protocols) smart contract, originating in July 2020. The NEARswap implemented a first version of this proposal in a form of multi token standard.

## Rationale

The only approved token standard in NEAR ecosystem is [NEP-21](https://github.com/near/NEPs/blob/master/specs/Standards/Tokens/FungibleToken.md).
It's an adaptation of [ERC-20](https://eips.ethereum.org/EIPS/eip-20) token standard from Ethereum. Both NEP-21 and ERC-20 are designed to be minimalistic and functional: provide clear interface for token transfers, allowance management (give access to token transfers to other entities or smart-contract - this is useful if other smart contract wants to withdraw it's allowance for things we buy). The ERC-20 standard is the first standard Ethereum fungible token standard and created lot of legacy. All early tooling (wallets, explorers, token templates) were build with ERC-20 in mind. Over time, the ecosystem listed many problems related to ERC-20 design:
1. `decimals` should be required, rather than optional. This is essential to user-friendly token amount handling.
2. Lack of standarized metdata reference. Token should provide a resource which describes it's metadata (name, description, etc...) - this can be an URL, CID, or directly written in the contract code.
3. `transfer` and `transferFrom` are lacking a reference (memo) argument - which is essential for compliance reasons. We need to be able to be able to link the transfer to an an external document (eg: invoice), order ID or other on-chain transaction or simply set a transfer reason.
4. Direct `transfers` to smart-contract in general is an error and should be protected. Both in Ethereum in NEAR, smart-contracts are NOT notified if someone is sending them tokens. This causes funds lost, token locked and many critical misbehavior.
5. Avoid problems mentioned in the previous point, all transfers should be done through `approve` (allowance creation) and `transferFrom`, which is less intuitive and makes UX more complex: not only we need to create and keep track of right allowance (with all edge cases: user creates allowance, but token is not calling `transferFrom` and makes user to create another allowance).
6. Fees calculation. With `approve` + `transferFrom`, the business provider has to make an additional transaction (transferFrom) and calculate it in the operation cost.

There are few articles analyzing ERC-20 flaws (and NEP-21): [What’s Wrong With ERC-20 Token?](https://ihodl.com/analytics/2018-10-04/whats-wrong-erc-20-token/), [Critical problems of ERC20 token standard](https://medium.com/@dexaran820/erc20-token-standard-critical-problems-3c10fd48657b).

And the NEP-110 discussion: https://github.com/near/NEPs/issues/110 addressing same issues in a bit different way.

### Proses of NEP-21 and ERC-20

The pay to smart-contract flow (`approve` + `tranferFrom`), even though it's not very user friendly and prone for wrong allowance, it's very simple. It moves a complexity of handling token-recipient interaction from the contract implementation to the recipient. This makes the contract design simpler and more secure in domains where reentrancy attack is possible.


### Related work

+ [NEP-110 Advanced Fungible Token Standard](https://github.com/near/NEPs/issues/110)
+ [NEP-122 Allowance-free vault-based token standard](https://github.com/near/NEPs/issues/122)
+ [ERC-20](https://eips.ethereum.org/EIPS/eip-20)
+ [ERC-223](https://github.com/Dexaran/ERC223-token-standard/blob/development/README.md)
+ [ERC-777](https://github.com/ethereum/EIPs/issues/777)

## Token design

We propose a new token standard to solve issues above. The design goals:
1. not trading off simplicity - the contract must be easy to implement and use
2. completely remove allowance: removes UX flaws and optimize contract storage space
3. simplify interaction with other smart-contracts
4. simplify flow in NEP-122
5. remove frictions related to different decimals


Our work is mostly influenced by the aforementioned [ERC-223](https://github.com/Dexaran/ERC223-token-standard/blob/development/README.md) and [NEP-122: Allowance-free vault-based token standard](https://github.com/near/NEPs/issues/122).

### Transfer reference (memo)

We add a required `memo` argument to all transfer functions. Similarly to bank transfer and payment orders, the `memo` argument allows to reference transfer to other event (on-chain or off-chain). It is a schema less, so user can use it to reference an external document, invoice, order ID, ticket ID, or other on-chain transaction. With `memo` you can set a transfer reason, often required for compliance.
This is also useful and very convenient for implementing FATA (Financial Action Task Force) [guidelines](http://www.fatf-gafi.org/media/fatf/documents/recommendations/RBA-VA-VASPs.pdf) (section 7(b) ). Especially a requirement for VASPs (Virtual Asset Service Providers) to collect and transfer customer information during transactions. VASP is any entity which provides to a user token custody, management, exchange or investment services. With ERC-20 (and NEP-21) it is not possible to do it in atomic way. With `memo` field, we can provide such reference in the same transaction and atomically bind it to the money transfer.

### Decimal management

Lack of decimals creates difficulty for handling user experience in all sorts of tools
As mentioned in the Rationale, for interactions with tools and assure good UX, we need to know the base of token arithmetic.
ERC-20 has an optional `decimals` contract attribute, other Ethereum standards makes this attribute obligatory. We follow the ERC-777 proposal, and fix the decimals once and for all:
+ Each token should have 18 digits precision (decimals), same as most of the existing Ethereum tokens. If a token contract returns a balance of 500,000,000,000,000,000 (0.5e18) for a user, the user interface MUST show 0.5 tokens to the user.
+ We port the `granularity` concept from ERC-777:  the smallest part of the token that’s (denominated in e18) not divisible. The following rules MUST be applied regarding the granularity:
+ The granularity value MUST be set at creation time.
+ The granularity value MUST NOT be changed, ever.
+ The granularity value MUST be greater than or equal to 1.
+ All balances MUST be a multiple of the granularity.
+ Any amount of tokens (in the internal denomination) minted, sent or burned MUST be a multiple of the granularity value.
+ Any operation that would result in a balance that’s not a multiple of the granularity value MUST be considered invalid, and the transaction MUST revert.

### Reactive transfers

Instead of having `approve` + `transferFrom`, we propose a `transfer_call` function which transfer funds and calls external smart-contract to notify him about the transfer. This function essentially requires that a recipient must implement `TransferCallRecipient` interface described below.


### Security note for `transfer_call`

In synchronous like environment (Ethereum EVM and all it's clones), reactive calls (like `transfer_call`, or `transfer` from ERC223) are susceptible for reentrancy attacks. In the discussion below lets denote a transaction for contract `A` which calls external smart contract `B`(we write `A->B`).
An attack vector is to call back the originating smart-contract (`B->A`) in the same transaction - we call it reentrancy. This creates various issues since the reentrance call is happening before all changes have been committed and it's not isolated from the originating call. This leads to many exploits which have been widely discussed and audited.

In asynchronous environment like NEAR, an external smart contract call execution is happening in a new, isolated routine once the originating call finished and all state changes have been committed.  This eliminates the reentrancy - any call from external smart contract back to the originating smart contract (`A->B->A`) is isolated from the originating smart-contract. The attack vector is limited and essentially it's "almost" reduced to other attacks happening in separate transaction. Almost - because a user still need to manage the callbacks.

### Handling not accepted calls

If a recipient of `transfer_call` fails, we would like to preserve the tokens from being lost. For that, a token MAY implement pattern developed by NEP-110:  to  sending tokens through `transfer_call`, append a `handle_token_received` callback promise to the `on_ft_receive` call. This callback will check if the previous one was successful, and if not, it will rollback the transfer.

You can check the NEP-110 `handle_token_received` [implementation](https://github.com/miohtama/advanced-fungible/blob/master/contract/token/src/token.rs#L351).


### Metadata

NEP-110 stores metadata on chain, and sets the final structure for the metadata:

```rust
struct Metadata {
  name: String,  // token name
  symbol: String, // token symbol
  web_link: String,  // URL to the human readable page about the token
  metadata_link: String, // URL to the metadata file with more information about the token, like different icon sets
}
```

We adopt this concept, but relax on it's content. Metadata should be as minimal as possible. We combine it with other token related data, and require that a contract will have following attributes:

```rust
struct Metadata {
  name: String,  // token name
  symbol: String, // token symbol
  reference: String, // URL to additional resources about the token.
  granularity: uint8,
  decimals = 18,
}
```


## Comparative analysis

We improve the token NEP-110 design by:
* handling compliance issues
* solving UX issues related to decimals
* clear support for smart-contract and basic transfers


We improve the NEP-122 design by:
* simplifying the flow (no need to create safe locks) and less callbacks
* handling compliance issues
* solving UX issues related to decimals

We improve the NEP-21 design by:
+ all points mentioned above
+ greatly simplifying implementation
+ reducing the storage size (no need to store allowances)
+ making the transfer interactive: being able to notify the recipient smart contract for the purchase / transfer.


## Token interface


Please look at the **[source code](./src/interface.rs)** for more details and comments.


```rust
struct Metadata {
    name: String,       // token name
    symbol: String,     // token symbol
    reference: String,  // URL to additional resources about the token.
    granularity: uint8, // the smallest part of the token that’s (denominated in e18) not divisible
    decimals: uint8,    // MUST be 18,
}

pub trait TransferCallRecipient {
    fn metadata() -> Metadata;
    fn total_supply(&self) -> U128;
    fn balance_of(&self, token: AccountId, holder: AccountId) -> U128;

    #[payable]
    fn transfer(&mut self, recipient: AccountId, amount: U128, msg: String, memo: String) -> bool;

    #[payable]
    fn transfer_call(
        &mut self,
        recipient: AccountId,
        amount: U128,
        msg: String,
        memo: String,
    ) -> bool;
}

pub trait TransferCallRecipient {
    fn on_ft_receive(
        &mut self,
        token: AccountId,
        from: AccountId,
        amount: U128,
        msg: String,
    ) -> bool;
}
```

## Further work

+ Extend this token standard for __operator_ functionality as defined in ERC-777. This should be backward compatible extension.
