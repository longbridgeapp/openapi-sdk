use std::sync::Arc;

use longbridge::{
    blocking::TradeContextSync,
    trade::{
        GetCashFlowOptions, GetFundPositionsOptions, GetHistoryExecutionsOptions,
        GetHistoryOrdersOptions, GetStockPositionsOptions, GetTodayExecutionsOptions,
        GetTodayOrdersOptions, ReplaceOrderOptions, SubmitOrderOptions,
    },
};
use pyo3::{pyclass, pymethods, PyObject, PyResult};

use crate::{
    config::Config,
    decimal::PyDecimal,
    time::{PyDateWrapper, PyOffsetDateTimeWrapper},
    trade::{
        push::handle_push_event,
        types::{
            AccountBalance, BalanceType, CashFlow, Execution, FundPositionsResponse, Order,
            OrderSide, OrderStatus, OrderType, OutsideRTH, StockPositionsResponse,
            SubmitOrderResponse, TimeInForceType, TopicType,
        },
    },
    types::Market,
};

#[pyclass]
pub(crate) struct TradeContext(TradeContextSync);

#[pymethods]
impl TradeContext {
    #[new]
    fn new(config: &Config, handler: Option<PyObject>) -> PyResult<Self> {
        let ctx = TradeContextSync::try_new(Arc::new(config.0.clone()), move |event| {
            if let Some(handler) = &handler {
                handle_push_event(handler, event);
            }
        })?;
        Ok(Self(ctx))
    }

    /// Subscribe
    fn subscribe(&self, topics: Vec<TopicType>) -> PyResult<()> {
        self.0.subscribe(topics.into_iter().map(Into::into))?;
        Ok(())
    }

    /// Unsubscribe
    fn unsubscribe(&self, topics: Vec<TopicType>) -> PyResult<()> {
        self.0.unsubscribe(topics.into_iter().map(Into::into))?;
        Ok(())
    }

    /// Get history executions
    pub fn history_executions(
        &self,
        symbol: Option<String>,
        start_at: Option<PyOffsetDateTimeWrapper>,
        end_at: Option<PyOffsetDateTimeWrapper>,
    ) -> PyResult<Vec<Execution>> {
        let mut opts = GetHistoryExecutionsOptions::new();

        if let Some(symbol) = symbol {
            opts = opts.symbol(symbol);
        }
        if let Some(start_at) = start_at {
            opts = opts.start_at(start_at.0);
        }
        if let Some(end_at) = end_at {
            opts = opts.end_at(end_at.0);
        }

        self.0
            .history_executions(Some(opts))?
            .into_iter()
            .map(TryInto::try_into)
            .collect()
    }

    /// Get today executions
    pub fn today_executions(
        &self,
        symbol: Option<String>,
        order_id: Option<String>,
    ) -> PyResult<Vec<Execution>> {
        let mut opts = GetTodayExecutionsOptions::new();

        if let Some(symbol) = symbol {
            opts = opts.symbol(symbol);
        }
        if let Some(order_id) = order_id {
            opts = opts.order_id(order_id);
        }

        self.0
            .today_executions(Some(opts))?
            .into_iter()
            .map(TryInto::try_into)
            .collect()
    }

    /// Get history orders
    #[args(default = "[]")]
    pub fn history_orders(
        &self,
        symbol: Option<String>,
        status: Vec<OrderStatus>,
        side: Option<OrderSide>,
        market: Option<Market>,
        start_at: Option<PyOffsetDateTimeWrapper>,
        end_at: Option<PyOffsetDateTimeWrapper>,
    ) -> PyResult<Vec<Order>> {
        let mut opts = GetHistoryOrdersOptions::new();

        if let Some(symbol) = symbol {
            opts = opts.symbol(symbol);
        }
        opts = opts.status(status.into_iter().map(Into::into));
        if let Some(side) = side {
            opts = opts.side(side.into());
        }
        if let Some(market) = market {
            opts = opts.market(market.into());
        }
        if let Some(start_at) = start_at {
            opts = opts.start_at(start_at.0);
        }
        if let Some(end_at) = end_at {
            opts = opts.end_at(end_at.0);
        }

        self.0
            .history_orders(Some(opts))?
            .into_iter()
            .map(TryInto::try_into)
            .collect()
    }

    /// Get today orders
    pub fn today_orders(
        &self,
        symbol: Option<String>,
        status: Vec<OrderStatus>,
        side: Option<OrderSide>,
        market: Option<Market>,
    ) -> PyResult<Vec<Order>> {
        let mut opts = GetTodayOrdersOptions::new();

        if let Some(symbol) = symbol {
            opts = opts.symbol(symbol);
        }
        opts = opts.status(status.into_iter().map(Into::into));
        if let Some(side) = side {
            opts = opts.side(side.into());
        }
        if let Some(market) = market {
            opts = opts.market(market.into());
        }

        self.0
            .today_orders(Some(opts))?
            .into_iter()
            .map(TryInto::try_into)
            .collect()
    }

    /// Replace order
    #[allow(clippy::too_many_arguments)]
    pub fn replace_order(
        &self,
        order_id: String,
        quantity: PyDecimal,
        price: Option<PyDecimal>,
        trigger_price: Option<PyDecimal>,
        limit_offset: Option<PyDecimal>,
        trailing_amount: Option<PyDecimal>,
        trailing_percent: Option<PyDecimal>,
        remark: Option<String>,
    ) -> PyResult<()> {
        let mut opts = ReplaceOrderOptions::new(order_id, quantity.into());

        if let Some(price) = price {
            opts = opts.price(price.into());
        }
        if let Some(trigger_price) = trigger_price {
            opts = opts.trigger_price(trigger_price.into());
        }
        if let Some(limit_offset) = limit_offset {
            opts = opts.limit_offset(limit_offset.into());
        }
        if let Some(trailing_amount) = trailing_amount {
            opts = opts.trailing_amount(trailing_amount.into());
        }
        if let Some(trailing_percent) = trailing_percent {
            opts = opts.trailing_percent(trailing_percent.into());
        }
        if let Some(remark) = remark {
            opts = opts.remark(remark);
        }

        self.0.replace_order(opts)?;
        Ok(())
    }

    /// Submit order
    #[allow(clippy::too_many_arguments)]
    pub fn submit_order(
        &self,
        symbol: String,
        order_type: OrderType,
        side: OrderSide,
        submitted_quantity: PyDecimal,
        time_in_force: TimeInForceType,
        submitted_price: Option<PyDecimal>,
        trigger_price: Option<PyDecimal>,
        limit_offset: Option<PyDecimal>,
        trailing_amount: Option<PyDecimal>,
        trailing_percent: Option<PyDecimal>,
        expire_date: Option<PyDateWrapper>,
        outside_rth: Option<OutsideRTH>,
        remark: Option<String>,
    ) -> PyResult<SubmitOrderResponse> {
        let mut opts = SubmitOrderOptions::new(
            symbol,
            order_type.into(),
            side.into(),
            submitted_quantity.into(),
            time_in_force.into(),
        );

        if let Some(submitted_price) = submitted_price {
            opts = opts.submitted_price(submitted_price.into());
        }
        if let Some(trigger_price) = trigger_price {
            opts = opts.trigger_price(trigger_price.into());
        }
        if let Some(limit_offset) = limit_offset {
            opts = opts.limit_offset(limit_offset.into());
        }
        if let Some(trailing_amount) = trailing_amount {
            opts = opts.trailing_amount(trailing_amount.into());
        }
        if let Some(trailing_percent) = trailing_percent {
            opts = opts.trailing_percent(trailing_percent.into());
        }
        if let Some(expire_date) = expire_date {
            opts = opts.expire_date(expire_date.0);
        }
        if let Some(outside_rth) = outside_rth {
            opts = opts.outside_rth(outside_rth.into());
        }
        if let Some(remark) = remark {
            opts = opts.remark(remark);
        }

        self.0.submit_order(opts)?.try_into()
    }

    /// Withdraw order
    pub fn withdraw_order(&self, order_id: String) -> PyResult<()> {
        self.0.withdraw_order(order_id)?;
        Ok(())
    }

    /// Get account balance
    pub fn account_balance(&self) -> PyResult<Vec<AccountBalance>> {
        self.0
            .account_balance()?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<PyResult<Vec<_>>>()
    }

    /// Get cash flow
    pub fn cash_flow(
        &self,
        start_at: PyOffsetDateTimeWrapper,
        end_at: PyOffsetDateTimeWrapper,
        business_type: Option<BalanceType>,
        symbol: Option<String>,
        page: Option<usize>,
        size: Option<usize>,
    ) -> PyResult<Vec<CashFlow>> {
        let mut opts = GetCashFlowOptions::new(start_at.0, end_at.0);

        if let Some(business_type) = business_type {
            opts = opts.business_type(business_type.into());
        }
        if let Some(symbol) = symbol {
            opts = opts.symbol(symbol);
        }
        if let Some(page) = page {
            opts = opts.page(page);
        }
        if let Some(size) = size {
            opts = opts.size(size);
        }

        self.0
            .cash_flow(opts)?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<PyResult<Vec<_>>>()
    }

    /// Get fund positions
    #[args(symbols = "vec![]")]
    pub fn fund_positions(&self, symbols: Vec<String>) -> PyResult<FundPositionsResponse> {
        self.0
            .fund_positions(GetFundPositionsOptions::new().symbols(symbols))?
            .try_into()
    }

    /// Get stock positions
    #[args(symbols = "vec![]")]
    pub fn stock_positions(&self, symbols: Vec<String>) -> PyResult<StockPositionsResponse> {
        self.0
            .stock_positions(GetStockPositionsOptions::new().symbols(symbols))?
            .try_into()
    }
}