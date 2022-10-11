interface GeneralInfo {
    multisig: string
}

interface AllowMsg {
    contract: string,
    gas_limit?: number,
}

interface CW20_ICS20 {
    admin: string,
    initMsg: {
        default_timeout: number,
        gov_contract: string,
        allowlist: AllowMsg[],
        default_gas_limit?: number,
    },
    label: string
}

interface IBCSatellite {
    admin: string,
    initMsg: {
        owner: string,
        astro_denom: string,
        transfer_channel: string,
        main_controller: string,
        main_maker: string,
        timeout: number
    },
    label: string
}

interface IBCController {
    admin: string,
    initMsg: {
        owner: string,
        assembly: string,
        timeout: number
    },
    label: string
}

interface Config {
    cw20_ics20: CW20_ICS20,
    controller: IBCController,
    satellite: IBCSatellite,
    generalInfo: GeneralInfo
}