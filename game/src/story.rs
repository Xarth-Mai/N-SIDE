//! The prologue's exterior handoff check, driven by real place observations
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const HANDOFF_PLACES: [&str; 3] = ["04", "29", "28"];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryRoute {
    #[default]
    ServiceYard,
    PublicSteps,
}

impl DeliveryRoute {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::ServiceYard => "配送走货运侧院",
            Self::PublicSteps => "配送走店侧台阶",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoffStage {
    Observing,
    AwaitingChoice,
    Confirmed,
}

impl HandoffStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Observing => "observing",
            Self::AwaitingChoice => "awaiting_choice",
            Self::Confirmed => "confirmed",
        }
    }
}

#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ShopHandoff {
    pub observed_places: BTreeSet<String>,
    pub confirmed_route: Option<DeliveryRoute>,
    pub rejected_choices: u32,
    pub confirmation_count: u32,
}

impl ShopHandoff {
    pub fn stage(&self) -> HandoffStage {
        if self.confirmed_route.is_some() {
            HandoffStage::Confirmed
        } else if HANDOFF_PLACES
            .iter()
            .all(|id| self.observed_places.contains(*id))
        {
            HandoffStage::AwaitingChoice
        } else {
            HandoffStage::Observing
        }
    }

    pub(crate) fn observe(&mut self, place: &str) {
        if HANDOFF_PLACES.contains(&place) && self.observed_places.insert(place.to_owned()) {
            info!("[story/observe] quest=QST-001 place={place}");
        }
    }

    pub(crate) fn can_choose_at(&self, place: Option<&str>) -> bool {
        place == Some("28") && self.stage() == HandoffStage::AwaitingChoice
    }

    pub(crate) fn choose(&mut self, place: Option<&str>, route: DeliveryRoute) {
        if !self.can_choose_at(place) {
            return;
        }
        match route {
            DeliveryRoute::ServiceYard => {
                self.confirmed_route = Some(route);
                self.confirmation_count += 1;
                info!("[story/commit] quest=QST-001 result=delivery_route route=service_yard");
            }
            DeliveryRoute::PublicSteps => {
                self.rejected_choices += 1;
                info!("[story/choice] quest=QST-001 route=public_steps result=revise");
            }
        }
    }

    pub(crate) fn objective(&self) -> &'static str {
        match self.stage() {
            HandoffStage::Confirmed => "路线已确认 · 配送走服务院，步行走公共台阶",
            HandoffStage::AwaitingChoice => "到货运侧院，选择本次交接路线",
            HandoffStage::Observing if !self.observed_places.contains("04") => {
                "到月台杂货门前，查看交接入口"
            }
            HandoffStage::Observing if !self.observed_places.contains("29") => {
                "到店侧台阶，核对步行近路"
            }
            HandoffStage::Observing => "到西侧货运侧院，核对配送入口",
        }
    }

    pub(crate) fn record(&self) -> String {
        let mut lines = vec!["门前交接 · 已观察".to_owned()];
        for (id, fact) in [
            ("04", "月台杂货：共用店门与西侧货门分开"),
            ("29", "店侧台阶：行人上坡近路，推车不能走台阶"),
            ("28", "货运侧院：配送经服务巷到达西侧货门"),
        ] {
            if self.observed_places.contains(id) {
                lines.push(fact.to_owned());
            }
        }
        if self.stage() == HandoffStage::Confirmed {
            lines.push("已记录：配送走服务院，步行走公共台阶".into());
        } else if self.rejected_choices > 0 {
            lines.push("需要修订：台阶可供步行，推车无法通过；请重新选择交接路线".into());
        } else {
            lines.push(self.objective().into());
        }
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observations_are_order_independent_and_choice_is_explicit_guarded_and_idempotent() {
        for order in [["04", "29", "28"], ["28", "04", "29"], ["29", "28", "04"]] {
            let mut state = ShopHandoff::default();
            state.observe("23");
            assert!(state.observed_places.is_empty());
            state.choose(Some("28"), DeliveryRoute::ServiceYard);
            assert_eq!(state.confirmation_count, 0);
            for id in order {
                state.observe(id);
                state.observe(id);
            }
            assert_eq!(state.observed_places.len(), 3);
            assert_eq!(state.stage(), HandoffStage::AwaitingChoice);
            assert_eq!(state.confirmation_count, 0, "visiting is not deciding");
            state.choose(Some("04"), DeliveryRoute::ServiceYard);
            assert_eq!(state.confirmation_count, 0);
            state.choose(Some("28"), DeliveryRoute::PublicSteps);
            assert_eq!(state.stage(), HandoffStage::AwaitingChoice);
            assert_eq!(state.rejected_choices, 1);
            assert!(state.record().contains("需要修订"));
            state.choose(Some("28"), DeliveryRoute::ServiceYard);
            state.choose(Some("28"), DeliveryRoute::ServiceYard);
            state.choose(Some("28"), DeliveryRoute::PublicSteps);
            assert_eq!(state.stage(), HandoffStage::Confirmed);
            assert_eq!(state.confirmation_count, 1);
            assert_eq!(state.rejected_choices, 1);
            assert!(!state.record().contains("需要修订"));
        }
    }
}
