use yew::prelude::*;

use crate::gui::RcUi;

#[derive(Clone)]
pub enum NumberType {
    Byte(RcUi<u8>),
    Integer(RcUi<i32>),
    Float(RcUi<f32>),
}

impl PartialEq for NumberType {
    fn eq(&self, other: &NumberType) -> bool {
        match self {
            NumberType::Byte(byte) => {
                if let NumberType::Byte(other) = other {
                    byte.ptr_eq(other)
                } else {
                    false
                }
            }
            NumberType::Integer(integer) => {
                if let NumberType::Integer(other) = other {
                    integer.ptr_eq(other)
                } else {
                    false
                }
            }
            NumberType::Float(float) => {
                if let NumberType::Float(other) = other {
                    float.ptr_eq(other)
                } else {
                    false
                }
            }
        }
    }
}

pub enum Msg {
    Change(ChangeData),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub label: String,
    pub value: NumberType,
}

pub struct InputNumber {
    props: Props,
    link: ComponentLink<Self>,
}

impl Component for InputNumber {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        InputNumber { props, link }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Change(data) => match data {
                ChangeData::Value(mut value) => {
                    if value.is_empty() {
                        value = String::from("0");
                    }

                    if let Ok(value) = value.parse::<f64>() {
                        match self.props.value {
                            NumberType::Byte(ref byte) => {
                                *byte.borrow_mut() = value as u8;
                            }
                            NumberType::Integer(ref integer) => {
                                *integer.borrow_mut() = value as i32
                            }
                            NumberType::Float(ref float) => *float.borrow_mut() = value as f32,
                        };
                    }
                    true
                }
                _ => false,
            },
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let Props { label, value } = &props;
        if self.props.label != *label || self.props.value != *value {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let (value, placeholder) = match self.props.value {
            NumberType::Byte(ref byte) => (byte.borrow().to_string(), "Byte"),
            NumberType::Integer(ref integer) => (integer.borrow().to_string(), "Integer"),
            NumberType::Float(ref float) => (float.borrow().to_string(), "Float"),
        };

        html! {
            <label class="flex items-center gap-1">
                <input type="number" class="input w-[120px]" placeholder=placeholder value=value onchange=self.link.callback(Msg::Change) />
                { &self.props.label }
            </label>
        }
    }
}
