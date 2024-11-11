import { type SendTransactionRequest, TonConnectUI, type WalletInfo, type WalletInfoCurrentlyInjected } from '@tonconnect/ui';
import { notification } from 'antd';
import { isMobile, openLink } from './device';
import { Cell } from '@ton/ton';

const dappMetadata = {
	manifestUrl:
		'https://overpass.network/tonconnect-manifest.json'
};

export const connector = new TonConnectUI(dappMetadata);

export async function sendTransaction(tx: SendTransactionRequest, wallet: WalletInfo): Promise<{ boc: string }> {
	try {
		if ('universalLink' in wallet && !(wallet as WalletInfoCurrentlyInjected).embedded && isMobile()) {
			openLink(addReturnStrategy(wallet.universalLink, 'none'), '_blank');
		}

		const txResult = await connector.sendTransaction(tx);
		notification.success({
			message: 'Successful transaction',
			description:
				'Your transaction was successfully sent. Please wait until the transaction is included in the TON blockchain.',
			duration: 5,
		});
		console.log(`Send tx result: ${JSON.stringify(txResult)}`);
		
		if (typeof txResult === 'object' && txResult !== null && 'boc' in txResult && txResult.boc) {
			console.log(`Send tx result: ${JSON.stringify(txResult)}`);
			return { boc: txResult.boc };
		} else {
			throw new Error('Transaction result does not contain a boc');
		}
	} catch (e) {
		let message = 'Send transaction error';
		let description = '';

		if (e instanceof Error && e.name === 'UserRejectError') {
			message = 'You rejected the transaction';
			description = 'Please try again and confirm transaction in your wallet.';
		}

		notification.error({
			message,
			description,
		});
		console.log(e);
		throw e;
	}
}
export function addReturnStrategy(url: string, returnStrategy: 'back' | 'none'): string {	const link = new URL(url);
	link.searchParams.append('ret', returnStrategy);
	return link.toString();
}