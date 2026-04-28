#![no_std]

mod events;
mod storage;
mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env};
use storage::*;
use types::ContractError;

#[contract]
pub struct TokenContract;

#[contractimpl]
impl TokenContract {
    /// Initialises the token contract, minting the entire initial supply to the admin.
    ///
    /// Must be called once before any other function.
    ///
    /// # Parameters
    /// - `env` – Soroban execution environment.
    /// - `admin` – Address that receives the initial supply and gains admin privileges.
    /// - `initial_supply` – Total tokens minted to `admin` at initialisation.
    pub fn initialize(env: Env, admin: Address, initial_supply: i128) {
        admin.require_auth();
        set_admin(&env, &admin);
        set_balance(&env, &admin, initial_supply);
        set_total_supply(&env, initial_supply);
        set_version(&env, (1, 0, 0));
    }

    /// Returns the total token supply in circulation.
    ///
    /// # Parameters
    /// - `env` – Soroban execution environment.
    ///
    /// # Returns
    /// Total supply as `i128`.
    pub fn total_supply(env: Env) -> i128 { total_supply(&env) }

    /// Returns the token balance of an address.
    ///
    /// # Parameters
    /// - `env` – Soroban execution environment.
    /// - `owner` – Address to query.
    ///
    /// # Returns
    /// Balance as `i128`. Returns `0` if the address has never held tokens.
    pub fn balance(env: Env, owner: Address) -> i128 { balance_of(&env, &owner) }

    /// Returns the token balance of an address (alias for [`balance`]).
    ///
    /// # Parameters
    /// - `env` – Soroban execution environment.
    /// - `owner` – Address to query.
    ///
    /// # Returns
    /// Balance as `i128`. Returns `0` if the address has never held tokens.
    pub fn balance_of(env: Env, owner: Address) -> i128 { balance_of(&env, &owner) }

    /// Transfers tokens from one address to another.
    ///
    /// # Parameters
    /// - `env` – Soroban execution environment.
    /// - `from` – Sender address; must authorise the call.
    /// - `to` – Recipient address.
    /// - `amount` – Number of tokens to transfer; must be positive.
    ///
    /// # Errors
    /// - [`ContractError::InvalidAmount`] if `amount` is zero or negative.
    /// - [`ContractError::InsufficientBalance`] if `from` has fewer tokens than `amount`.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), ContractError> {
        from.require_auth();
        if amount <= 0 { return Err(ContractError::InvalidAmount); }
        // Transfer to self is a no-op: auth is still required but no state changes occur.
        if from == to { return Ok(()); }
        let b = balance_of(&env, &from);
        if b < amount { return Err(ContractError::InsufficientBalance); }
        set_balance(&env, &from, b - amount);
        set_balance(&env, &to, balance_of(&env, &to) + amount);
        events::transferred(&env, &from, &to, amount);
        Ok(())
    }

    /// Approves `spender` to transfer up to `amount` tokens on behalf of `owner`.
    ///
    /// Overwrites any existing allowance. Stored in temporary storage (expires with the ledger).
    ///
    /// # Parameters
    /// - `env` – Soroban execution environment.
    /// - `owner` – Token owner granting the allowance; must authorise the call.
    /// - `spender` – Address permitted to spend on behalf of `owner`.
    /// - `amount` – Maximum tokens the spender may transfer.
    pub fn approve(env: Env, owner: Address, spender: Address, amount: i128) {
        owner.require_auth();
        set_allowance(&env, &owner, &spender, amount);
    }

    /// Transfers tokens on behalf of `from` using a pre-approved allowance.
    ///
    /// # Parameters
    /// - `env` – Soroban execution environment.
    /// - `spender` – Address spending the allowance; must authorise the call.
    /// - `from` – Token owner whose balance is debited.
    /// - `to` – Recipient address.
    /// - `amount` – Number of tokens to transfer; must not exceed the allowance.
    ///
    /// # Errors
    /// - [`ContractError::AllowanceExceeded`] if `amount` exceeds the current allowance.
    /// - [`ContractError::InsufficientBalance`] if `from` has fewer tokens than `amount`.
    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) -> Result<(), ContractError> {
        spender.require_auth();
        let allowed = allowance(&env, &from, &spender);
        if allowed < amount { return Err(ContractError::AllowanceExceeded); }
        let b = balance_of(&env, &from);
        if b < amount { return Err(ContractError::InsufficientBalance); }
        set_allowance(&env, &from, &spender, allowed - amount);
        set_balance(&env, &from, b - amount);
        set_balance(&env, &to, balance_of(&env, &to) + amount);
        events::transferred(&env, &from, &to, amount);
        Ok(())
    }

    /// Mints new tokens to an address, increasing the total supply.
    ///
    /// Only the admin may mint tokens.
    ///
    /// # Parameters
    /// - `env` – Soroban execution environment.
    /// - `admin` – Admin address; must authorise the call.
    /// - `to` – Address that receives the newly minted tokens.
    /// - `amount` – Number of tokens to mint; must be positive.
    ///
    /// # Errors
    /// - [`ContractError::NotAdmin`] if `admin` does not match the stored admin.
    /// - [`ContractError::InvalidAmount`] if `amount` is zero or negative.
    pub fn mint(env: Env, admin: Address, to: Address, amount: i128) -> Result<(), ContractError> {
        admin.require_auth();
        if get_admin(&env)? != admin { return Err(ContractError::NotAdmin); }
        if amount <= 0 { return Err(ContractError::InvalidAmount); }
        set_balance(&env, &to, balance_of(&env, &to) + amount);
        set_total_supply(&env, total_supply(&env) + amount);
        events::minted(&env, &to, amount);
        Ok(())
    }

    /// Burns tokens from an address, reducing the total supply.
    ///
    /// Only the admin may burn tokens.
    ///
    /// # Parameters
    /// - `env` – Soroban execution environment.
    /// - `admin` – Admin address; must authorise the call.
    /// - `from` – Address whose tokens are burned.
    /// - `amount` – Number of tokens to burn.
    ///
    /// # Errors
    /// - [`ContractError::NotAdmin`] if `admin` does not match the stored admin.
    /// - [`ContractError::InsufficientBalance`] if `from` has fewer tokens than `amount`.
    pub fn burn(env: Env, admin: Address, from: Address, amount: i128) -> Result<(), ContractError> {
        admin.require_auth();
        if get_admin(&env)? != admin { return Err(ContractError::NotAdmin); }
        let b = balance_of(&env, &from);
        if b < amount { return Err(ContractError::InsufficientBalance); }
        set_balance(&env, &from, b - amount);
        set_total_supply(&env, total_supply(&env) - amount);
        events::burned(&env, &from, amount);
        Ok(())
    }

    /// Transfers admin rights to a new address. Only the current admin may call this.
    ///
    /// The old admin loses all privileges immediately upon successful transfer.
    ///
    /// # Parameters
    /// - `env` – Soroban execution environment.
    /// - `admin` – Current admin address; must authorise the call.
    /// - `new_admin` – Address that will become the new admin.
    ///
    /// # Errors
    /// - [`ContractError::NotAdmin`] if `admin` does not match the stored admin.
    /// - [`ContractError::InvalidNewAdmin`] if `new_admin` is the zero address.
    pub fn transfer_admin(
        env: Env,
        admin: Address,
        new_admin: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        if get_admin(&env)? != admin {
            return Err(ContractError::NotAdmin);
        }
        let zero = Address::from_str(
            &env,
            "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
        );
        if new_admin == zero {
            return Err(ContractError::InvalidNewAdmin);
        }
        set_admin(&env, &new_admin);
        events::admin_transferred(&env, &admin, &new_admin);
        Ok(())
    }

    /// Returns the contract version as a `(major, minor, patch)` semver tuple.
    pub fn get_version(env: Env) -> (u32, u32, u32) {
        get_version(&env)
    }
}
