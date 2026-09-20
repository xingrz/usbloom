//! OS hotplug notifications and coalescing for asynchronous descriptor scans.

/// Register before the initial scan so a device change cannot fall into the
/// gap between enumeration and observation. Registration does not open devices.
pub fn watch() -> Result<nusb::hotplug::HotplugWatch, nusb::Error> {
    nusb::watch_devices()
}

/// Keep one scan in flight and remember changes that arrive during it.
#[derive(Default)]
pub struct ScanGate {
    generation: u64,
    active: bool,
    pending: bool,
}

impl ScanGate {
    pub fn is_scanning(&self) -> bool {
        self.active
    }

    pub fn request(&mut self) -> Option<u64> {
        if self.active {
            self.pending = true;
            return None;
        }
        self.generation += 1;
        self.active = true;
        Some(self.generation)
    }

    /// None rejects a stale result; true requests one follow-up scan.
    pub fn complete(&mut self, generation: u64) -> Option<bool> {
        if !self.active || self.generation != generation {
            return None;
        }
        self.active = false;
        Some(std::mem::take(&mut self.pending))
    }

    /// Entering an offline snapshot invalidates both in-flight and queued work.
    pub fn invalidate(&mut self) {
        self.generation += 1;
        self.active = false;
        self.pending = false;
    }
}

#[cfg(test)]
mod tests {
    use super::ScanGate;

    #[test]
    fn burst_during_a_scan_requests_exactly_one_followup() {
        let mut gate = ScanGate::default();
        let initial = gate.request().unwrap();
        for _ in 0..20 {
            assert_eq!(gate.request(), None);
        }
        assert_eq!(gate.complete(initial), Some(true));
        let followup = gate.request().unwrap();
        assert_eq!(gate.complete(followup), Some(false));
        assert!(!gate.is_scanning());
    }

    #[test]
    fn snapshot_rejects_old_results_and_discards_pending_changes() {
        let mut gate = ScanGate::default();
        let old = gate.request().unwrap();
        gate.request();
        gate.invalidate();
        assert_eq!(gate.complete(old), None);
        let live = gate.request().unwrap();
        assert_eq!(gate.complete(old), None);
        assert!(gate.is_scanning());
        assert_eq!(gate.complete(live), Some(false));
    }

    #[test]
    fn a_finished_scan_does_not_schedule_periodic_work() {
        let mut gate = ScanGate::default();
        let initial = gate.request().unwrap();
        assert_eq!(gate.complete(initial), Some(false));
        assert_eq!(gate.complete(initial), None);
        assert!(!gate.is_scanning());
    }
}
