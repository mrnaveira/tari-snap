export type SetWalletParams = {
    tari_wallet_daemon_url: string;
};

export type TariPermission = Object;

export type SendWalletRequestParams = {
    token: string,
    walletRequest: WalletRequest
};

export type WalletRequest = {
    method: string,
    params: Object
};

export type IndexerRequest = {
    method: string,
    params: Object
};