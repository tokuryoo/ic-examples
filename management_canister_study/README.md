# management_canister_study
キャニスターがキャニスターを作る実装例。

```bash
dfx start --background
# management_canister_study_backend をデプロイ（メインネットではなくローカル）
dfx deploy

# management_canister_study_backend が保持している Cycle の確認（メインネットではなくローカル）
dfx canister status management_canister_study_backend
# management_canister_study_backend へ Cycle を発行（メインネットではなくローカル）
dfx ledger fabricate-cycles --canister management_canister_study_backend --cycles 8000000000000
# management_canister_study_backend が保持している Cycle の再確認（メインネットではなくローカル）
dfx canister status management_canister_study_backend

# 動作確認
dfx canister call management_canister_study_backend create_canister_example
dfx canister call management_canister_study_backend create_canister_example2
dfx canister call management_canister_study_backend create_canister_example3

```

詳しくは lib.rs のコメントを参照。