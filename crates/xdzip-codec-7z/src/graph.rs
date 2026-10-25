use crate::source::Budget;
use crate::types::{Bind, Coder, Folder, MethodId, PackedInput};
use crate::wire::Cursor;
use xdzip_codec_api::{Cancellation, ScanError};
const MAX_CODERS: usize = 64;
const MAX_PORTS: usize = 256;

pub(crate) fn folder(
    c: &mut Cursor<'_>,
    budget: &mut Budget,
    control: &Cancellation,
) -> Result<Folder, ScanError> {
    let count = c.count(MAX_CODERS)?;
    if count == 0 {
        return Err(ScanError::Invalid);
    }
    let mut coders = budget.vec(count)?;
    let (mut inputs, mut outputs) = (0usize, 0usize);
    let mut input_owner = Vec::new();
    let mut output_owner = Vec::new();
    for owner in 0..count {
        control.check()?;
        let flags = c.byte()?;
        if flags & 0xc0 != 0 {
            return Err(ScanError::Unsupported);
        }
        let id_length = usize::from(flags & 15);
        budget.claim(id_length)?;
        let method = MethodId::new(c.take(id_length)?.to_vec())?;
        let (n_in, n_out) = if flags & 0x10 == 0 {
            (1, 1)
        } else {
            (c.count(MAX_PORTS)?, c.count(MAX_PORTS)?)
        };
        if n_in == 0 || n_out == 0 {
            return Err(ScanError::Invalid);
        }
        inputs = inputs
            .checked_add(n_in)
            .filter(|n| *n <= MAX_PORTS)
            .ok_or(ScanError::Limit)?;
        outputs = outputs
            .checked_add(n_out)
            .filter(|n| *n <= MAX_PORTS)
            .ok_or(ScanError::Limit)?;
        budget.claim((n_in + n_out) * std::mem::size_of::<usize>() * 2)?;
        input_owner.extend(std::iter::repeat_n(owner, n_in));
        output_owner.extend(std::iter::repeat_n(owner, n_out));
        let properties = if flags & 0x20 != 0 {
            let n = c.count(64 * 1024)?;
            budget.claim(n)?;
            c.take(n)?.to_vec()
        } else {
            Vec::new()
        };
        coders.push(Coder {
            method,
            properties,
            inputs: n_in,
            outputs: n_out,
        });
    }
    let bound = outputs.checked_sub(1).ok_or(ScanError::Invalid)?;
    let packed_count = inputs
        .checked_sub(bound)
        .filter(|n| *n != 0)
        .ok_or(ScanError::Invalid)?;
    budget.claim(inputs + outputs)?;
    let (mut input_used, mut output_used) = (vec![false; inputs], vec![false; outputs]);
    let mut binds = budget.vec(bound)?;
    let mut edges: Vec<Vec<usize>> = budget.vec(count)?;
    edges.resize_with(count, Vec::new);
    budget.claim(bound * std::mem::size_of::<usize>() * 2)?;
    let mut indegree = budget.vec(count)?;
    indegree.resize(count, 0usize);
    for _ in 0..bound {
        let input = usize::try_from(c.number()?).map_err(|_| ScanError::Invalid)?;
        let output = usize::try_from(c.number()?).map_err(|_| ScanError::Invalid)?;
        if input >= inputs || output >= outputs || input_used[input] || output_used[output] {
            return Err(ScanError::Invalid);
        }
        input_used[input] = true;
        output_used[output] = true;
        let (producer, consumer) = (output_owner[output], input_owner[input]);
        edges[producer].push(consumer);
        indegree[consumer] += 1;
        binds.push(Bind { input, output });
    }
    let terminal_output = output_used
        .iter()
        .position(|used| !used)
        .ok_or(ScanError::Invalid)?;
    if output_used.iter().filter(|used| !**used).count() != 1 {
        return Err(ScanError::Invalid);
    }
    let mut packed_inputs = budget.vec(packed_count)?;
    for _ in 0..packed_count {
        let input = if packed_count == 1 {
            input_used
                .iter()
                .position(|used| !used)
                .ok_or(ScanError::Invalid)?
        } else {
            usize::try_from(c.number()?).map_err(|_| ScanError::Invalid)?
        };
        if input >= inputs || input_used[input] {
            return Err(ScanError::Invalid);
        }
        input_used[input] = true;
        packed_inputs.push(PackedInput { input, pack: 0 });
    }
    if input_used.iter().any(|used| !*used) {
        return Err(ScanError::Invalid);
    }
    let mut order = budget.vec(count)?;
    for (i, degree) in indegree.iter().enumerate() {
        if *degree == 0 {
            order.push(i);
        }
    }
    let mut at = 0;
    while at < order.len() {
        control.check()?;
        for next in &edges[order[at]] {
            indegree[*next] -= 1;
            if indegree[*next] == 0 {
                order.push(*next);
            }
        }
        at += 1;
    }
    if order.len() != count {
        return Err(ScanError::Invalid);
    }
    // All outputs except the terminal have one consumer. In an acyclic graph
    // with nonempty port sets this also establishes terminal reachability.
    Ok(Folder {
        coders,
        binds,
        packed_inputs,
        output_sizes: Vec::new(),
        terminal_output,
        crc: None,
        substreams: Vec::new(),
        topological_order: order,
    })
}
