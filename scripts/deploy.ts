import 'dotenv/config'
import {
    newClient,
    writeArtifact,
    readArtifact,
    deployContract,
} from './helpers.js'
import { join } from 'path'
import { LCDClient } from '@terra-money/terra.js';
import { chainConfigs } from "./types.d/chain_configs.js";

const ARTIFACTS_PATH = '../artifacts'

async function main() {
    const { terra, wallet } = newClient()
    console.log(`chainID: ${terra.config.chainID} wallet: ${wallet.key.accAddress}`)

    if (!chainConfigs.generalInfo.multisig) {
        throw new Error("Set the proper owner multisig for the contracts")
    }

    await uploadAndInitICS20(terra, wallet)
    await uploadAndInitController(terra, wallet)
}

async function uploadAndInitICS20(terra: LCDClient, wallet: any) {
    let network = readArtifact(terra.config.chainID)

    if (!network.cw20Ics20Address) {
        chainConfigs.cw20_ics20.admin ||= chainConfigs.generalInfo.multisig
        chainConfigs.cw20_ics20.initMsg.gov_contract ||= chainConfigs.generalInfo.multisig

        console.log('Deploying CW20-ICS20...')
        let resp = await deployContract(
            terra,
            wallet,
            chainConfigs.cw20_ics20.admin,
            join(ARTIFACTS_PATH, 'astroport_cw20_ics20.wasm'),
            chainConfigs.cw20_ics20.initMsg,
            chainConfigs.cw20_ics20.label,
        )

        // @ts-ignore
        network.cw20Ics20Address = resp.shift().shift()
        console.log("cw20-ics20:", network.cw20Ics20Address)
        writeArtifact(network, terra.config.chainID)
    }
}

async function uploadAndInitController(terra: LCDClient, wallet: any) {
    let network = readArtifact(terra.config.chainID)

    if (!network.ibcControllerAddress) {

        if (!chainConfigs.controller.initMsg.assembly) {
            throw new Error("Please deploy the Assembly contract")
        }

        chainConfigs.controller.initMsg.owner ||= chainConfigs.generalInfo.multisig
        chainConfigs.controller.admin ||= chainConfigs.generalInfo.multisig

        console.log('Deploying IBC Controller...')
        let resp = await deployContract(
            terra,
            wallet,
            chainConfigs.controller.admin,
            join(ARTIFACTS_PATH, 'ibc_controller.wasm'),
            chainConfigs.controller.initMsg,
            chainConfigs.controller.label,
        )

        // @ts-ignore
        network.ibcControllerAddress = resp.shift().shift()
        console.log(`IBC Controller Contract Address: ${network.ibcControllerAddress}`)
        writeArtifact(network, terra.config.chainID)
    }
}

await main()
