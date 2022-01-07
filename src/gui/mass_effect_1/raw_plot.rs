use std::rc::Rc;

use yew::prelude::*;

use crate::{
    gui::{
        components::{Tab, TabBar},
        shared::{FloatPlotType, IntPlotType, PlotType, RawPlot},
        RcUi,
    },
    save_data::shared::plot::BitVec,
    services::database::Databases,
};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub booleans: RcUi<BitVec>,
    pub integers: IntPlotType,
    pub floats: FloatPlotType,
}

#[function_component(Me1RawPlot)]
pub fn me1_raw_plot(props: &Props) -> Html {
    let dbs = use_context::<Databases>().expect("no database provider");
    if let Some(ref plot_db) = dbs.get_me1_raw_plot() {
        let (booleans, integers, floats) = (&props.booleans, &props.integers, &props.floats);

        html! {
            <TabBar>
                <Tab title="Booleans">
                    <RawPlot plots={PlotType::Boolean(RcUi::clone(booleans))} plot_db={Rc::clone(plot_db)} />
                </Tab>
                <Tab title="Integers">
                    <RawPlot plots={PlotType::Int(integers.clone())} plot_db={Rc::clone(plot_db)} />
                </Tab>
                <Tab title="Floats">
                    <RawPlot plots={PlotType::Float(floats.clone())} plot_db={Rc::clone(plot_db)} />
                </Tab>
            </TabBar>
        }
    } else {
        html! {
            <>
                <p>{ "Loading database..." }</p>
                <hr class="border-t border-default-border" />
            </>
        }
    }
}
