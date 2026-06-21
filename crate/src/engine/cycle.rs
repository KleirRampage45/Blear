use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClickCycleKind { Single, Double }

#[derive(Clone, Copy, Debug)]
pub struct ClickCyclePlan {
    pub kind: ClickCycleKind,
    pub first_hold_ms: u32,
    pub inter_click_gap_ms: u32,
    pub second_hold_ms: u32,
}

impl ClickCyclePlan {
    pub fn single(hold_ms: u32) -> Self {
        Self { kind: ClickCycleKind::Single, first_hold_ms: hold_ms, inter_click_gap_ms: 0, second_hold_ms: 0 }
    }

    pub fn double(requested_hold_ms: u32, cycle_ms: u32, inter_click_gap_ms: u32) -> Self {
        let clamped_gap_ms = inter_click_gap_ms.min(cycle_ms.saturating_sub(1));
        let second_hold_ms = requested_hold_ms.min(cycle_ms.saturating_sub(clamped_gap_ms));
        Self { kind: ClickCycleKind::Double, first_hold_ms: 0, inter_click_gap_ms: clamped_gap_ms, second_hold_ms }
    }
}

pub fn execute_click_cycle<FPress, FRelease, FSleep, FActive>(
    plan: ClickCyclePlan,
    press: &mut FPress,
    release: &mut FRelease,
    sleep_for: &mut FSleep,
    is_active: &FActive,
) -> bool
where
    FPress: FnMut(),
    FRelease: FnMut(),
    FSleep: FnMut(Duration),
    FActive: Fn() -> bool,
{
    if !dispatch_press_release(plan.first_hold_ms, press, release, sleep_for, is_active) {
        return false;
    }

    if plan.kind == ClickCycleKind::Double {
        if plan.inter_click_gap_ms > 0 {
            sleep_for(Duration::from_millis(plan.inter_click_gap_ms as u64));
            if !is_active() { return false; }
        }
        return dispatch_press_release(plan.second_hold_ms, press, release, sleep_for, is_active);
    }

    true
}

fn dispatch_press_release<FPress, FRelease, FSleep, FActive>(
    hold_ms: u32,
    press: &mut FPress,
    release: &mut FRelease,
    sleep_for: &mut FSleep,
    is_active: &FActive,
) -> bool
where
    FPress: FnMut(),
    FRelease: FnMut(),
    FSleep: FnMut(Duration),
    FActive: Fn() -> bool,
{
    if !is_active() { return false; }
    press();
    if hold_ms > 0 {
        sleep_for(Duration::from_millis(hold_ms as u64));
        if !is_active() { release(); return false; }
    }
    release();
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_click_plan() {
        let plan = ClickCyclePlan::single(50);
        assert_eq!(plan.kind, ClickCycleKind::Single);
        assert_eq!(plan.first_hold_ms, 50);
        assert_eq!(plan.inter_click_gap_ms, 0);
    }

    #[test]
    fn test_double_click_plan() {
        let plan = ClickCyclePlan::double(100, 500, 150);
        assert_eq!(plan.kind, ClickCycleKind::Double);
        assert_eq!(plan.inter_click_gap_ms, 150);
        assert_eq!(plan.second_hold_ms, 100);
    }

    #[test]
    fn test_double_click_gap_clamped() {
        // gap_ms > cycle_ms - 1 should be clamped
        let plan = ClickCyclePlan::double(100, 200, 500);
        assert_eq!(plan.inter_click_gap_ms, 199);
    }

    #[test]
    fn test_execute_single_cycle() {
        let mut press_count = 0;
        let mut release_count = 0;
        let result = execute_click_cycle(
            ClickCyclePlan::single(0),
            &mut || press_count += 1,
            &mut || release_count += 1,
            &mut |_| {},
            &|| true,
        );
        assert!(result);
        assert_eq!(press_count, 1);
        assert_eq!(release_count, 1);
    }

    #[test]
    fn test_execute_double_cycle() {
        let mut press_count = 0;
        let mut release_count = 0;
        let result = execute_click_cycle(
            ClickCyclePlan::double(0, 500, 100),
            &mut || press_count += 1,
            &mut || release_count += 1,
            &mut |_| {},
            &|| true,
        );
        assert!(result);
        assert_eq!(press_count, 2);
        assert_eq!(release_count, 2);
    }

    #[test]
    fn test_execute_aborted_when_inactive() {
        let mut press_count = 0;
        let mut release_count = 0;
        // Pre-set inactive
        let result = execute_click_cycle(
            ClickCyclePlan::single(50),
            &mut || press_count += 1,
            &mut || release_count += 1,
            &mut |_| {},
            &|| false,
        );
        assert!(!result);
        assert_eq!(press_count, 0);
        assert_eq!(release_count, 0);
    }
}
