loop {
    let mut attempted_liquidation = false;
    let mut failed_liquidation = false;

    if !is_first_iteration {
        tokio::time::sleep(Duration::from_millis(60_000)).await;
    } else {
        is_first_iteration = false;
    }

    let trgs = get_trgs_in_mpg(
        admin.ctx.dex_program_id,
        args.market_product_group,
        &admin.authority,
        network.clone(),
    )
    .await;

    match admin.ctx.load_products().await {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to load products with error: {:?}", e);
            continue;
        }
    }

    let mpg_pubkey = args.market_product_group;
    let market_product_group = admin
        .ctx
        .client
        .get_anchor_account::<MarketProductGroup>(mpg_pubkey)
        .await;
    .expect("Failed to load MarketProductGroup account");

    if market_product_group.state != MarketProductGroupState::Active {
        error!(
            "MarketProductGroup is not active, state: {}",
            market_product_group.state as u8
        );
        continue;
    }

    // Get other required anchor accounts for a successful liquidation
    let admin_market_product_groups =
        admin
            .ctx
            .client
            .get_admin_market_product_groups(admin.admin_pubkey)
            .await
            .expect("Failed to get admin market product groups");

    let mut liquidateable_currencies: Vec<(&Currency, &Account)> = vec![];

    for currency in market_product_group.currencies.iter() {
        let pubkey = currency.1;
        let market_pubkey = currency.0;
        let market = admin
            .ctx
            .client
            .get_anchor_account::<Market>(market_pubkey)
            .await
            .expect("Failed to load market");
        if market.state != MarketState::Active {
            error!(
                "Market {} is not active, state: {}",
                market_pubkey,
                market.state as u8
            );
            continue;
        }

        let oracle = admin
            .ctx
            .client
            .get_anchor_account::<Oracle>(market.oracle)
            .await
            .expect("Failed to load Oracle");

        if oracle.state != OracleState::Active {
            error!(
                "Oracle is not active, state: {}",
                oracle.state as u8
            );
            continue;
        }

        let currency = admin
            .ctx
            .client
            .get_anchor_account::<Account>(pubkey)
            .await
            .expect("Failed to load Currency account");

        if currency.state != AccountState::Active {
            error!(
                "Currency account is not active, state: {}",
                currency.state as u8
            );
            continue;
        }

        let latest_price = oracle
            .get_latest_price()
            .await
            .expect("Failed to get latest price");

        if latest_price.is_none() {
            error!("Latest price is None");
            continue;
        }

        let latest_price = latest_price.unwrap();
        info!(
            "Latest price for {} is {}",
            market_pubkey,
            latest_price.price
        );

        let pending_requests = market
            .get_pending_requests()
            .await
            .expect("Failed to get pending requests");

        if pending_requests.is_empty() {
            info!("No pending requests for {}", market_pubkey);
            continue;
        }

        for request in pending_requests.iter() {
            let user_pubkey = &request.user_pubkey;
            let amount = request.amount;
            let price_limit = request.price_limit;

            if latest_price.price <= price_limit {
                info!(
                    "Processing request for user {} with price limit {}",
                    user_pubkey,
                    price_limit
                );

                let transaction_result = market
                    .execute_order(user_pubkey, amount, latest_price.price)
                    .await;

                match transaction_result {
                    Ok(_) => {
                        info!(
                            "Successfully executed order for user {}",
                            user_pubkey
                        );
                    }
                    Err(e) => {
                        error!(
                            "Failed to execute order for user {}: {}",
                            user_pubkey,
                            e
                        );
                    }
                }
            } else {
                info!(
                    "Skipping request for user {} as current price {} is higher than price limit {}",
                    user_pubkey,
                    latest_price.price,
                    price_limit
                );
            }
        }
                // Remove processed requests from the queue
                pending_requests.retain(|request| latest_price.price > request.price_limit);

                // Sleep for a specified interval before fetching new data
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        }
        
        #[tokio::main]
        async fn main() -> Result<(), Box<dyn std::error::Error>> {
            let market = Market::new();
        
            // Initialize the logger for better debugging and monitoring
            SimpleLogger::new().with_level(log::LevelFilter::Info).init().unwrap();
        
            // Start the market maker service
            start_market_maker_service(&market).await?;
        
            Ok(())
        }
        
        
        
        
        
        
        