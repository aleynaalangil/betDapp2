#[solana_sdk::program]
pub fn bet_on_event(
    instruction_data: &bet_on_event::Instruction,
    account_keys: &bet_on_event::AccountKeys,
    _program_id: &solana_sdk::pubkey::Pubkey,
    _lamports: &mut solana_sdk::lamports::Lamports,
    _data: &[u8],
) -> solana_sdk::program_error::ProgramError {
    if instruction_data.event_id.len() != 32 {
        return solana_sdk::program_error::ProgramError::InvalidArgument;
    }

    let event_account_pubkey = solana_sdk::pubkey::new_rand();
    let event_account_info = solana_sdk::sysvar::rent::get_account_rent_info(&event_account_pubkey);

    match instruction_data.instruction_type {
        bet_on_event::InstructionType::PlaceBet => {
            if event_account_info.status != bet_on_event::EventStatus::Active {
                return solana_sdk::program_error::ProgramError::InvalidArgument;
            }
            if *_lamports < instruction_data.bet_amount {
                return solana_sdk::program_error::ProgramError::InsufficientFunds;
            }
            let account_data = match solana_sdk::storage::read(&event_account_pubkey) {
                Ok(data) => data,
                Err(_) => return solana_sdk::program_error::ProgramError::InvalidArgument,
            };
            let event: Event = match bincode::deserialize(&account_data) {
                Ok(event) => event,
                Err(_) => return solana_sdk::program_error::ProgramError::InvalidArgument,
            };
            let mut account = match solana_sdk::bank::get_account(&account_keys.pubkey) {
                Ok(account) => account,
                Err(_) => return solana_sdk::program_error::ProgramError::InvalidArgument,
            };
            let mut bet = match account.get_data() {
                Some(data) => data,
                None => return solana_sdk::program_error::ProgramError::InvalidArgument,
            };
            if bet.status != bet_on_event::BetStatus::Open {
                return solana_sdk::program_error::ProgramError::InvalidArgument;
            }
            bet.event_id = instruction_data.event_id;
            bet.bet_amount = instruction_data.bet_amount;
            bet.bet_option = instruction_data.bet_option;
            bet.status = bet_on_event::BetStatus::Placed;
            account.set_data(bet);
            _lamports -= instruction_data.bet_amount;
            solana_sdk::bank::store_account(&account_keys.pubkey, &account);
        }
        bet_on_event::InstructionType::CollectWinnings => {
            let event_account_data = match solana_sdk::storage::read(&instruction_data.event_id) {
                Ok(data) => data,
                Err(_) => return solana_sdk::program_error::ProgramError::InvalidArgument,
            };
            let event: Event = match bincode::deserialize(&event_account_data) {
                Ok(event) => event,
                Err(_) => return solana_sdk::program_error::ProgramError::InvalidArgument,
            };
            if event.status != bet_on_event::EventStatus::Complete {
                return solana_sdk::program_error::ProgramError::InvalidArgument;
            }
            let mut account = match solana_sdk::bank::get_account(&account_keys.pubkey) {
                Ok(account) => account,
                Err(_) => return solana_sdk::program_error::ProgramError::InvalidArgument,
            };
            let bet = match account.get_data() {
                Some(data) => data,
                None => return solana_sdk::program_error::ProgramError::InvalidArgument,
            };
            if bet.status != bet_on_event::BetStatus::Placed {
                return solana_sdk::program_error::ProgramError::InvalidArgument;
            }
            if bet.event_id != instruction_data.event_id {
                return solana_sdk::program_error::ProgramError::InvalidArgument;
            }
            if bet.bet_option != event.winning_option {
                return solana_sdk::program_error::ProgramError::InvalidArgument;
            }
            *_lamports += bet.bet_amount;
            bet.status = bet_on_event::BetStatus::Collected;
            account.set_data(bet);
            solana_sdk::sysvar::rent::put_account(&account_keys.pubkey, &account);
        }
    }
    solana_sdk::program_error::ProgramError::Success;}