use super::*;

struct Work {
    maximum: u64,
    done: u64,
    stop_after: Option<u64>,
    stopped_kind: io::ErrorKind,
}
impl Work {
    fn unlimited() -> Self {
        Self {
            maximum: u64::MAX,
            done: 0,
            stop_after: None,
            stopped_kind: io::ErrorKind::Other,
        }
    }
}
impl PpmdWork for Work {
    fn checkpoint(&mut self) -> io::Result<()> {
        if self
            .stop_after
            .is_some_and(|threshold| self.done >= threshold)
        {
            Err(io::Error::new(
                self.stopped_kind,
                "injected_terminal_work_error",
            ))
        } else {
            Ok(())
        }
    }
    fn reserve_initialization(&mut self, bytes: u64) -> io::Result<()> {
        self.checkpoint()?;
        if bytes > self.maximum.saturating_sub(self.done) {
            return Err(io::ErrorKind::OutOfMemory.into());
        }
        Ok(())
    }
    fn initialized(&mut self, bytes: u64) -> io::Result<()> {
        self.done += bytes;
        Ok(())
    }
}
#[test]
fn ppmd_initialization_admits_before_work_and_records_only_completed_chunks() {
    let mut work = Work::unlimited();
    let model = PpmdModel::with_work(64, 1 << 20, PpmdCapacity::Rar, &mut work).unwrap();
    let initialization = work.done;
    assert!(initialization > model.allocated_bytes());
    let mut work = Work {
        maximum: initialization - 1,
        ..Work::unlimited()
    };
    assert!(PpmdModel::with_work(64, 1 << 20, PpmdCapacity::Rar, &mut work).is_err());
    assert_eq!(work.done, 0, "future reservation cannot become actual WORK");
    for kind in [io::ErrorKind::Other, io::ErrorKind::OutOfMemory] {
        let chunk = (65536 / std::mem::size_of::<arena::Cell>()
            * std::mem::size_of::<arena::Cell>()) as u64;
        let mut work = Work {
            stop_after: Some(chunk),
            stopped_kind: kind,
            ..Work::unlimited()
        };
        let error = PpmdModel::with_work(64, 1 << 20, PpmdCapacity::Rar, &mut work)
            .err()
            .unwrap();
        assert_eq!(error.kind(), kind);
        assert_eq!(
            work.done, chunk,
            "only the first completed chunk survives cancellation/OOM"
        );
    }
}

// A statistical selection driver, not a range encoder or compressed stream.
// It chooses legal probability intervals to exercise deterministic capacity.
struct Selections {
    work: Work,
    random: u32,
}
impl Selections {
    fn next(&mut self) -> u32 {
        self.random = self.random.wrapping_mul(1664525).wrapping_add(1013904223);
        self.random
    }
}
impl PpmdWork for Selections {
    fn checkpoint(&mut self) -> io::Result<()> {
        self.work.checkpoint()
    }
    fn reserve_initialization(&mut self, bytes: u64) -> io::Result<()> {
        self.work.reserve_initialization(bytes)
    }
    fn initialized(&mut self, bytes: u64) -> io::Result<()> {
        self.work.initialized(bytes)
    }
}
impl PpmdRange for Selections {
    fn count(&mut self, total: u32) -> io::Result<(u32, u32)> {
        Ok((self.next() % (total - 1), 1))
    }
    fn select(&mut self, _unit: u32, _start: u32, _frequency: u32) -> io::Result<()> {
        Ok(())
    }
    fn binary(&mut self, probability: u16) -> io::Result<bool> {
        Ok(self.next() % 16384 < u32::from(probability))
    }
}
#[test]
fn ppmd_rar_capacity_restarts_report_actual_work_and_do_not_recover_host_failure() {
    let mut work = Work::unlimited();
    let mut model = PpmdModel::with_work(64, 2048, PpmdCapacity::Rar, &mut work).unwrap();
    let initial = work.done;
    let mut selections = Selections { work, random: 42 };
    for _ in 0..4096 {
        assert!(model.decode(&mut selections).unwrap().is_some());
    }
    let diagnostics = model.diagnostics();
    assert!(diagnostics.restarts > 10);
    let completed = selections.work.done;
    let restart = (completed - initial) / diagnostics.restarts;
    assert_eq!(completed, initial + restart * diagnostics.restarts);
    assert!(restart > 2048);
    selections.work.maximum = completed + restart - 1;
    let mut rejected = false;
    for _ in 0..4096 {
        if let Err(error) = model.decode(&mut selections) {
            assert_eq!(error.kind(), io::ErrorKind::OutOfMemory);
            rejected = true;
            break;
        }
    }
    assert!(rejected);
    assert_eq!(
        selections.work.done, completed,
        "failed restart admission must report no imaginary initialization"
    );
}
