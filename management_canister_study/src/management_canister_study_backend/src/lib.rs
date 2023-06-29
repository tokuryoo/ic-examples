use ic_cdk::api::call::RejectionCode;
use ic_cdk::api::management_canister::main::{
    canister_status, create_canister, delete_canister, deposit_cycles, install_code,
    start_canister, stop_canister, uninstall_code, update_settings, CanisterIdRecord,
    CanisterInstallMode, CanisterSettings, CanisterStatusType, CreateCanisterArgument,
    InstallCodeArgument, UpdateSettingsArgument,
};
use ic_cdk::{export::candid::Principal, update};

// 参考 https://github.com/dfinity/cdk-rs/tree/main/examples/management_canister

// キャニスターの作成、インストール、アンインストール、削除のサンプルコード。コントローラーは management_canister_study_backend 自身。
// 実行コマンド例
// $ dfx canister call management_canister_study_backend create_canister_example
#[update]
async fn create_canister_example() {
    let create_canister_arg = CreateCanisterArgument {
        settings: Some(CanisterSettings {
            // controllers でコントローラーを指定できる。
            // https://docs.rs/ic-cdk/latest/ic_cdk/api/fn.id.html
            // ic_cdk::id() - 自身のキャニスターID（Principal）
            controllers: Some(vec![ic_cdk::id()]),
            compute_allocation: Some(0.into()),
            memory_allocation: Some(10000.into()),
            freezing_threshold: Some(10000.into()),
        }),
    };

    // dfx canister create 相当。空のキャニスターが作られる。
    // Cycle が消費されるので注意。
    // https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-create_canister
    // https://docs.rs/ic-cdk/latest/ic_cdk/api/management_canister/main/fn.create_canister.html
    // ic-cdk 0.7 では 引数に Cycle が無いので注意。今回は、ic-cdk 0.9
    // 最低 100_000_000_000 Cycle 必要。しかし、少なすぎると、すぐ枯渇してエラーになると予想される（検証不足）。
    // https://mora.app/planet/nhdmt-lqaaa-aaaan-qddra-cai/7TKESG8YK4PQ763X892Q7V98HJ
    let canister_id_records = create_canister(create_canister_arg, 200_000_000_000)
        .await
        .unwrap();
    let canister_id_record = canister_id_records.0;
    let canister_id = canister_id_record.canister_id;

    // 上記で確保した空のキャニスター（確保したキャニスターID）へインストール
    // https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-install_code
    let install_code_arg = InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id,
        wasm_module: b"\x00asm\x01\x00\x00\x00".to_vec(), // 固定の WASM モジュール
        arg: vec![],
    };

    // https://docs.rs/ic-cdk/latest/ic_cdk/api/management_canister/main/fn.install_code.html
    // install_code(install_code_arg).await.unwrap(); // unwrap() しているため、Err の場合、パニックになる。
    let call_result = install_code(install_code_arg).await;
    if let Err(e) = call_result {
        // 必要に応じてエラー処理
        let (_rejection_code, _s): (RejectionCode, String) = e;
    }
    // 以降、エラー処理は省き、unwrap() とする。

    let canister_id_record = CanisterIdRecord { canister_id };

    // 上記でインストールしたキャニスターの起動
    // https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-start_canister
    start_canister(canister_id_record).await.unwrap();

    // 上記でインストールしたキャニスターの停止。処理中ものがが全て処理された後、停止されて、応答が返ってくるとのこと。
    // https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-stop_canister
    stop_canister(canister_id_record).await.unwrap();
    // 停止されたかどうか確認
    let response = canister_status(canister_id_record).await.unwrap().0;
    assert_eq!(response.status, CanisterStatusType::Stopped);

    // management_canister_study_backend 自身 から 上記でインストールしたキャニスター へ Cycle を送る
    deposit_cycles(canister_id_record, 1_000_000_000_000u128)
        .await
        .unwrap();

    // 上記でインストールしたキャニスターのアンインストール（空のキャニスターになる）
    uninstall_code(canister_id_record).await.unwrap();

    // キャニスターを削除。
    // 削除するにあたり、あらかじめ停止しておく必要があるとのこと。
    // 削除すると、そのキャニスターIDは再利用できない。
    // Cycle は破棄されると書かれている。
    // https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-delete_canister
    delete_canister(canister_id_record).await.unwrap();
}

// キャニスターを作成する。ただし、そのキャニスターのコントローラーは、クライアントの Princiapl にする。
// 戻り値は、作成されたキャニスターのキャニスターID（Principal）
// 実行例
// $ dfx identity get-principal
// c6oov-ntjsg-mpt26-plmiq-h7dtr-6mpwx-g4qka-piapl-hc6gv-yhf77-qae
// $ dfx canister call management_canister_study_backend create_canister_example2
// (principal "aax3a-h4aaa-aaaaa-qaahq-cai")
// $ dfx canister info aax3a-h4aaa-aaaaa-qaahq-cai
// Controllers: c6oov-ntjsg-mpt26-plmiq-h7dtr-6mpwx-g4qka-piapl-hc6gv-yhf77-qae
// Module hash: None
// 上記の例では、コントローラーがクライアントの Principal となっていることを確認できている。
#[update]
async fn create_canister_example2() -> Principal {
    // 最初に principal を取得しないと、実行時にエラーが発生するので注意。
    let principal: Principal = ic_cdk::api::caller();
    let create_canister_arg = CreateCanisterArgument {
        settings: Some(CanisterSettings {
            controllers: Some(vec![principal]),
            compute_allocation: Some(0.into()),
            memory_allocation: Some(10000.into()),
            freezing_threshold: Some(10000.into()),
        }),
    };
    let canister_id_records = create_canister(create_canister_arg, 200_000_000_000)
        .await
        .unwrap();
    let canister_id_record = canister_id_records.0;
    let canister_id = canister_id_record.canister_id;

    // コントローラーが自身ではないので、パニックになる。
    // let install_code_arg = InstallCodeArgument {
    //     mode: CanisterInstallMode::Install,
    //     canister_id,
    //     wasm_module: b"\x00asm\x01\x00\x00\x00".to_vec(), // 固定の WASM モジュール
    //     arg: vec![],
    // };
    // install_code(install_code_arg).await.unwrap();

    // コントローラーが自身ではないので、パニックになる。
    // 別途、該当の Principal で dfx canister start を行う必要がある。
    // start_canister(canister_id_record).await.unwrap();

    // コントローラーが自身ではないので、パニックになる。
    // let response = canister_status(canister_id_record).await.unwrap().0;

    canister_id
}

// キャニスターを作成する。そのキャニスターのコントローラーは、management_canister_study_backend 自身とする。インストールした後（自身がコントローラーなのでインストールできる）、コントローラーをクライアントの Princiapl に変更している。（あくまでサンプル実装であり、冗長である）
// 戻り値は、作成されたキャニスターのキャニスターID（Principal）
// 実行例
// $ dfx identity get-principal
// c6oov-ntjsg-mpt26-plmiq-h7dtr-6mpwx-g4qka-piapl-hc6gv-yhf77-qae
// $ dfx canister call management_canister_study_backend create_canister_example3
// (principal "d6g4o-amaaa-aaaaa-qaaoq-cai")
// $ dfx canister info d6g4o-amaaa-aaaaa-qaaoq-cai
// Controllers: c6oov-ntjsg-mpt26-plmiq-h7dtr-6mpwx-g4qka-piapl-hc6gv-yhf77-qae
// Module hash: 0x93a44bbb96c751218e4c00d479e4c14358122a389acca16205b1e4d0dc5f9476
// 上記の例では、コントローラーがクライアントの Principal に変更されたことを確認できている。
#[update]
async fn create_canister_example3() -> Principal {
    let principal: Principal = ic_cdk::api::caller();
    let create_canister_arg = CreateCanisterArgument {
        settings: Some(CanisterSettings {
            // ic_cdk::id() - 自身のキャニスターID（Principal）
            controllers: Some(vec![ic_cdk::id()]),
            compute_allocation: None, // デフォルトは 0
            memory_allocation: None,  // デフォルトは 0
            freezing_threshold: None, // デフォルトは 2592000（約30日）。30 日後に Cycle が無くなると予想される場合、凍結されるらしい？
        }),
    };
    // https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-create_canister
    let canister_id_records = create_canister(create_canister_arg, 200_000_000_000)
        .await
        .unwrap();
    let canister_id_record = canister_id_records.0;
    let canister_id = canister_id_record.canister_id;

    let install_code_arg = InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id,
        wasm_module: b"\x00asm\x01\x00\x00\x00".to_vec(), // 固定の WASM モジュール
        arg: vec![],
    };
    install_code(install_code_arg).await.unwrap();

    // https://docs.rs/ic-cdk/latest/ic_cdk/api/management_canister/main/struct.UpdateSettingsArgument.html
    let update_settings_argument = UpdateSettingsArgument {
        // canister_id: canister_id, // 同じ名前なので、Rust では下記のように短縮して書くことができる
        canister_id,
        settings: CanisterSettings {
            controllers: Some(vec![principal]), // コントローラーをクライアントに変更
            compute_allocation: None,           // デフォルトは 0
            memory_allocation: None,            // デフォルトは 0
            freezing_threshold: None, // デフォルトは 2592000（約30日）。30 日後に Cycle が無くなると予想される場合、凍結される？
        },
    };

    // https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-update_settings
    // https://docs.rs/ic-cdk/latest/ic_cdk/api/management_canister/main/fn.update_settings.html
    update_settings(update_settings_argument).await.unwrap();

    canister_id
}
