import { UserSecretKey } from '@elrondnetwork/erdjs';
import * as fs from 'fs';

const file = fs.readFileSync('./test-signer.pem').toString();
const privateKey = UserSecretKey.fromPem(file);

const publicKey = privateKey.generatePublicKey();

console.log('public key hex', publicKey.hex());

// my_address: 6d795f616464726573735F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F
// other_address: 6f746865725f616464726573735F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F

const signature = privateKey.sign(
  Buffer.concat([
    Buffer.from('6d795f616464726573735F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F', 'hex'),
    Buffer.from('nft-create-uri13'),
  ])
);

const signatureHex = signature.toString('hex');

console.log('signature ', signatureHex);

console.log(
  'verifying signature',
  publicKey.verify(
    Buffer.concat([
      Buffer.from('6d795f616464726573735F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F', 'hex'),
      Buffer.from('nft-create-uri13'),
    ]),
    Buffer.from(signatureHex, 'hex')
  )
);