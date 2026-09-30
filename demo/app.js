// EigenTrust playground: an editable trust network ranked live by the WASM engine.

import { LANGUAGES, detectLanguage, saveLanguage, setLanguage, lang, t, num } from './i18n.js'

const MAX_GRAPH_PEERS = 400
const NAMES = ['alice', 'bob', 'carol', 'dave', 'erin', 'frank', 'grace', 'heidi', 'ivan', 'judy',
    'mallory', 'niaj', 'olivia', 'peggy', 'rupert', 'trent', 'victor', 'walter', 'yara', 'zoe']

const $ = (id) => document.getElementById(id)
const canvas = $('graph')
const ctx = canvas.getContext('2d')

// ---------- state ----------

const state = {
    peers: new Map(),   // id -> { id, x, y, vx, vy, r, score, group }
    edges: new Map(),   // "from\u0000to" -> { from, to, w }
    seeds: new Set(),
    alpha: 0.5,
    selected: null,
    hover: null,
    scores: [],         // [[peer, score]] from the last run, sorted
    large: null,        // { lt, pt, peers } when the network is too big to draw
    error: null,        // engine error for the current network
    lastRun: null,      // { ms, mode }
}

const edgeKey = (from, to) => from + '\u0000' + to

function nextName() {
    for (const n of NAMES) if (!state.peers.has(n)) return n
    let i = state.peers.size + 1
    while (state.peers.has('peer' + i)) i++
    return 'peer' + i
}

function addPeer(id = nextName(), x, y, group) {
    const { w, h } = size()
    if (x === undefined) {
        x = w / 2 + (Math.random() - 0.5) * 160
        y = h * 0.56 + (Math.random() - 0.5) * 160
    }
    state.peers.set(id, { id, x, y, vx: 0, vy: 0, r: 7, score: 0, group })
    return id
}

function removePeer(id) {
    state.peers.delete(id)
    state.seeds.delete(id)
    for (const [k, e] of state.edges) if (e.from === id || e.to === id) state.edges.delete(k)
    if (state.selected === id) state.selected = null
    changed()
}

function setEdge(from, to, w) {
    if (from === to) return
    if (!(w > 0)) state.edges.delete(edgeKey(from, to))
    else state.edges.set(edgeKey(from, to), { from, to, w })
    changed()
}

function toggleSeed(id) {
    if (state.seeds.has(id)) state.seeds.delete(id)
    else state.seeds.add(id)
    changed()
}

function clearNetwork() {
    state.peers.clear()
    state.edges.clear()
    state.seeds.clear()
    state.selected = null
    state.large = null
    state.scores = []
    state.error = null
}

// ---------- presets ----------

const PRESETS = {
    friends: {
        seeds: ['alice'],
        edges: [
            ['alice', 'bob', 3], ['alice', 'carol', 2], ['bob', 'carol', 1], ['bob', 'dave', 2],
            ['carol', 'alice', 1], ['carol', 'erin', 2], ['dave', 'bob', 1], ['dave', 'frank', 1],
            ['erin', 'carol', 1], ['erin', 'grace', 3], ['frank', 'dave', 2], ['grace', 'erin', 1],
            ['grace', 'heidi', 1], ['heidi', 'ivan', 2], ['ivan', 'judy', 1], ['judy', 'heidi', 1],
            ['heidi', 'grace', 1],
        ],
    },
    sybil: (() => {
        const honest = ['alice', 'bob', 'carol', 'dave', 'erin', 'frank']
        const sybils = Array.from({ length: 8 }, (_, i) => 'sybil' + (i + 1))
        const edges = [
            ['alice', 'bob', 2], ['bob', 'alice', 2], ['alice', 'carol', 1], ['carol', 'dave', 2],
            ['dave', 'alice', 1], ['bob', 'erin', 1], ['erin', 'frank', 2], ['frank', 'carol', 1],
            ['carol', 'bob', 1], ['dave', 'erin', 1],
            ['frank', 'sybil1', 1],
        ]
        for (const a of sybils) for (const b of sybils) if (a !== b) edges.push([a, b, 5])
        return { seeds: ['alice', 'bob'], edges, groups: { sybil: sybils }, alpha: 0.2 }
    })(),
    chain: {
        seeds: ['alice'],
        edges: ['alice', 'bob', 'carol', 'dave', 'erin', 'frank', 'grace', 'heidi']
            .map((n, i, a) => i < a.length - 1 ? [n, a[i + 1], 1] : null).filter(Boolean),
    },
    empty: { seeds: [], edges: [] },
}

function loadNetwork({ edges, seeds, groups = {}, alpha }) {
    clearNetwork()
    const groupOf = new Map()
    for (const [g, ids] of Object.entries(groups)) for (const id of ids) groupOf.set(id, g)
    const { w, h } = size()
    const ids = [...new Set(edges.flatMap(([a, b]) => [a, b]).concat(seeds))]
    ids.forEach((id, i) => {
        const angle = (i / ids.length) * Math.PI * 2
        const rad = Math.min(w, h) * 0.28
        addPeer(id, w / 2 + Math.cos(angle) * rad, h * 0.56 + Math.sin(angle) * rad, groupOf.get(id))
    })
    for (const [a, b, wt] of edges) if (a !== b && wt > 0) state.edges.set(edgeKey(a, b), { from: a, to: b, w: wt })
    for (const s of seeds) if (state.peers.has(s)) state.seeds.add(s)
    if (alpha !== undefined) setAlpha(alpha)
    reheat(1)
    changed()
}

// ---------- engine ----------

let worker
let jobId = 0
const jobs = new Map()

function startWorker() {
    worker = new Worker('worker.js', { type: 'module' })
    worker.onmessage = ({ data }) => {
        const job = jobs.get(data.id)
        jobs.delete(data.id)
        if (data.fatal) {
            worker.terminate()
            startWorker()
        }
        if (job) job(data)
    }
}
startWorker()

function runEngine(localtrust, pretrust, alpha) {
    const id = ++jobId
    return new Promise((resolve) => {
        jobs.set(id, resolve)
        const transfer = [localtrust, pretrust].filter((x) => x instanceof Uint8Array).map((x) => x.buffer)
        worker.postMessage({ id, localtrust, pretrust, alpha }, transfer)
    })
}

function networkCsv() {
    const lt = []
    const linked = new Set()
    for (const e of state.edges.values()) {
        lt.push(`${csvField(e.from)},${csvField(e.to)},${e.w}`)
        linked.add(e.from)
        linked.add(e.to)
    }
    const pt = [...state.seeds].filter((s) => linked.has(s)).map((s) => `${csvField(s)},1`)
    return { lt: lt.join('\n'), pt: pt.join('\n') }
}

let running = false
let dirty = false

function changed() {
    dirty = true
    if (!running) compute()
    renderPanel()
    writeHash()
}

async function compute() {
    dirty = false
    if (!state.large && state.edges.size === 0) {
        state.scores = []
        applyScores()
        return
    }
    running = true
    const { lt, pt } = state.large || networkCsv()
    const res = await runEngine(lt, pt, state.alpha)
    running = false
    state.error = res.error || null
    if (res.error) {
        state.scores = []
        state.lastRun = null
    } else {
        state.scores = res.scores
        state.lastRun = { ms: res.ms, threads: res.threads }
    }
    if (res.threads) $('engine').textContent = engineText()
    applyScores()
    if (dirty) compute()
}

const fmtMs = (ms) => ms < 1 ? num(ms, { maximumFractionDigits: 2 }) : ms < 10 ? num(ms, { maximumFractionDigits: 1 }) : num(Math.round(ms))
const modeText = (threads) => t('mode', { n: num(threads || 1) })

function engineText() {
    if (!state.lastRun) return t('running')
    return t('rankedIn', { ms: fmtMs(state.lastRun.ms), mode: modeText(state.lastRun.threads) })
}

function applyScores() {
    const byId = new Map(state.scores)
    for (const p of state.peers.values()) p.score = byId.get(p.id) || 0
    renderPanel()
    wake()
}

// ---------- layout simulation ----------

let heat = 1

function reheat(v = 0.5) {
    heat = Math.max(heat, v)
    wake()
}

function size() {
    const r = canvas.getBoundingClientRect()
    return { w: r.width || 800, h: r.height || 600 }
}

function step() {
    const peers = [...state.peers.values()]
    const { w, h } = size()
    const cx = w / 2
    const cy = h * 0.58
    const n = peers.length
    const springLength = Math.max(90, Math.min(190, Math.min(w, h) / 5))

    for (let i = 0; i < n; i++) {
        const a = peers[i]
        for (let j = i + 1; j < n; j++) {
            const b = peers[j]
            let dx = a.x - b.x
            let dy = a.y - b.y
            let d2 = dx * dx + dy * dy
            if (d2 < 0.01) { dx = Math.random() - 0.5; dy = Math.random() - 0.5; d2 = 0.5 }
            const minD = a.r + b.r + 18
            const f = (springLength * springLength * 0.25 + minD * minD * 2) / d2
            const d = Math.sqrt(d2)
            a.vx += (dx / d) * f; a.vy += (dy / d) * f
            b.vx -= (dx / d) * f; b.vy -= (dy / d) * f
        }
    }
    for (const e of state.edges.values()) {
        const a = state.peers.get(e.from)
        const b = state.peers.get(e.to)
        const dx = b.x - a.x
        const dy = b.y - a.y
        const d = Math.sqrt(dx * dx + dy * dy) || 1
        const f = (d - springLength) * 0.035
        a.vx += (dx / d) * f; a.vy += (dy / d) * f
        b.vx -= (dx / d) * f; b.vy -= (dy / d) * f
    }
    const pad = 28
    for (const p of peers) {
        if (p === drag?.peer) { p.vx = p.vy = 0; continue }
        p.vx += (cx - p.x) * 0.012
        p.vy += (cy - p.y) * 0.016
        p.x += Math.max(-30, Math.min(30, p.vx * heat))
        p.y += Math.max(-30, Math.min(30, p.vy * heat))
        p.vx *= 0.55
        p.vy *= 0.55
        p.x = Math.max(pad, Math.min(w - pad, p.x))
        p.y = Math.max(pad, Math.min(h - pad, p.y))
    }
    heat *= 0.985
}

// ---------- drawing ----------

let colors = {}
function readColors() {
    const cs = getComputedStyle(document.documentElement)
    for (const k of ['ink', 'muted', 'line', 'edge', 'trust', 'seed', 'surface', 'bg']) colors[k] = cs.getPropertyValue('--' + k).trim()
}
readColors()
matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => { readColors(); wake() })

const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)').matches
let frame = 0
let pointer = null

function wake() {
    if (!frame) frame = requestAnimationFrame(tick)
}

function tick() {
    frame = 0
    const graphShown = !state.large && state.peers.size <= MAX_GRAPH_PEERS
    let moving = false
    if (graphShown) {
        if (heat > 0.02) { step(); moving = true }
        const max = Math.max(1e-9, ...[...state.peers.values()].map((p) => p.score))
        for (const p of state.peers.values()) {
            const target = 7 + 24 * Math.sqrt(p.score / max)
            if (reducedMotion) p.r = target
            else p.r += (target - p.r) * 0.18
            if (Math.abs(target - p.r) > 0.05) moving = true
        }
    }
    draw(graphShown)
    if (moving || drag) wake()
}

function resize() {
    const dpr = window.devicePixelRatio || 1
    const { w, h } = size()
    canvas.width = Math.round(w * dpr)
    canvas.height = Math.round(h * dpr)
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
    wake()
}
new ResizeObserver(resize).observe(canvas)

function edgeGeometry(a, b) {
    const dx = b.x - a.x
    const dy = b.y - a.y
    const d = Math.sqrt(dx * dx + dy * dy) || 1
    // bend every edge to its right so a<->b pairs stay apart
    const bend = Math.min(26, d * 0.14)
    const mx = (a.x + b.x) / 2 - (dy / d) * bend
    const my = (a.y + b.y) / 2 + (dx / d) * bend
    const sx = mx - a.x, sy = my - a.y, sl = Math.hypot(sx, sy) || 1
    const ex = b.x - mx, ey = b.y - my, el = Math.hypot(ex, ey) || 1
    return {
        x1: a.x + (sx / sl) * (a.r + 2), y1: a.y + (sy / sl) * (a.r + 2),
        cx: mx, cy: my,
        x2: b.x - (ex / el) * (b.r + 4), y2: b.y - (ey / el) * (b.r + 4),
        ux: ex / el, uy: ey / el,
    }
}

function arrow(g, width) {
    ctx.beginPath()
    ctx.moveTo(g.x1, g.y1)
    ctx.quadraticCurveTo(g.cx, g.cy, g.x2, g.y2)
    ctx.lineWidth = width
    ctx.stroke()
    const s = 5 + width * 1.2
    ctx.beginPath()
    ctx.moveTo(g.x2 + g.ux * 2, g.y2 + g.uy * 2)
    ctx.lineTo(g.x2 - g.ux * s - g.uy * s * 0.55, g.y2 - g.uy * s + g.ux * s * 0.55)
    ctx.lineTo(g.x2 - g.ux * s + g.uy * s * 0.55, g.y2 - g.uy * s - g.ux * s * 0.55)
    ctx.closePath()
    ctx.fill()
}

function draw(graphShown) {
    const { w, h } = size()
    ctx.clearRect(0, 0, w, h)
    if (!graphShown) return

    const sel = state.selected
    const maxW = Math.max(1, ...[...state.edges.values()].map((e) => e.w))

    for (const e of state.edges.values()) {
        const a = state.peers.get(e.from)
        const b = state.peers.get(e.to)
        const hot = sel && (e.from === sel || e.to === sel)
        ctx.strokeStyle = ctx.fillStyle = hot ? colors.trust : colors.edge
        ctx.globalAlpha = sel && !hot ? 0.35 : 0.9
        arrow(edgeGeometry(a, b), 1 + 2.5 * (e.w / maxW))
    }
    ctx.globalAlpha = 1

    // preview of the next trust link
    if (sel && state.peers.has(sel) && pointer && !drag) {
        const a = state.peers.get(sel)
        const target = state.hover && state.hover !== sel ? state.peers.get(state.hover) : null
        ctx.setLineDash([4, 5])
        ctx.strokeStyle = ctx.fillStyle = colors.trust
        ctx.globalAlpha = 0.7
        if (target) arrow(edgeGeometry(a, target), 1.5)
        else {
            ctx.beginPath()
            ctx.moveTo(a.x, a.y)
            ctx.lineTo(pointer.x, pointer.y)
            ctx.lineWidth = 1.2
            ctx.stroke()
        }
        ctx.setLineDash([])
        ctx.globalAlpha = 1
    }

    const max = Math.max(1e-9, ...[...state.peers.values()].map((p) => p.score))
    const ranked = new Set(state.scores.slice(0, 12).map(([id]) => id))
    const showAll = state.peers.size <= 40

    for (const p of state.peers.values()) {
        const share = p.score / max
        ctx.beginPath()
        ctx.arc(p.x, p.y, p.r, 0, Math.PI * 2)
        ctx.fillStyle = colors.surface
        ctx.fill()
        ctx.globalAlpha = p.score > 0 ? 0.22 + 0.78 * share : 0.12
        ctx.fillStyle = colors.trust
        ctx.fill()
        ctx.globalAlpha = 1
        if (state.seeds.has(p.id)) {
            ctx.lineWidth = 3
            ctx.strokeStyle = colors.seed
            ctx.stroke()
        }
        if (p.id === sel || p.id === state.hover) {
            ctx.beginPath()
            ctx.arc(p.x, p.y, p.r + (state.seeds.has(p.id) ? 6 : 4), 0, Math.PI * 2)
            ctx.lineWidth = 1.5
            ctx.strokeStyle = colors.ink
            ctx.stroke()
        }
        if (showAll || ranked.has(p.id) || p.id === sel || p.id === state.hover) {
            ctx.font = `${p.id === sel ? 600 : 500} 12.5px "Schibsted Grotesk", system-ui, sans-serif`
            ctx.textAlign = 'center'
            ctx.textBaseline = 'top'
            ctx.lineWidth = 4
            ctx.strokeStyle = colors.bg
            ctx.strokeText(p.id, p.x, p.y + p.r + 5)
            ctx.fillStyle = p.score > 0 ? colors.ink : colors.muted
            ctx.fillText(p.id, p.x, p.y + p.r + 5)
        }
    }
}

// ---------- pointer interaction ----------
// tap a peer: select it; tap another: add trust from the selected peer to it.
// double tap a peer: toggle seed. double tap empty space: add a peer. drag: move.

let drag = null
let lastTap = { id: undefined, t: 0 }
let pendingTap = 0

function localPoint(ev) {
    const r = canvas.getBoundingClientRect()
    return { x: ev.clientX - r.left, y: ev.clientY - r.top }
}

function peerAt(pt) {
    let best = null
    let bestD = Infinity
    for (const p of state.peers.values()) {
        const d = Math.hypot(p.x - pt.x, p.y - pt.y)
        if (d <= Math.max(p.r, 12) + 4 && d < bestD) { best = p; bestD = d }
    }
    return best
}

canvas.addEventListener('pointerdown', (ev) => {
    if (state.large) return
    const pt = localPoint(ev)
    const peer = peerAt(pt)
    drag = { peer, start: pt, moved: false }
    canvas.setPointerCapture(ev.pointerId)
})

canvas.addEventListener('pointermove', (ev) => {
    pointer = localPoint(ev)
    if (drag?.peer) {
        if (Math.hypot(pointer.x - drag.start.x, pointer.y - drag.start.y) > 5) drag.moved = true
        if (drag.moved) {
            drag.peer.x = pointer.x
            drag.peer.y = pointer.y
            reheat(0.3)
        }
    } else {
        const hover = peerAt(pointer)?.id || null
        if (hover !== state.hover) state.hover = hover
        canvas.style.cursor = hover ? 'pointer' : 'default'
    }
    wake()
})

canvas.addEventListener('pointerleave', () => { pointer = null; state.hover = null; wake() })

canvas.addEventListener('pointerup', (ev) => {
    const d = drag
    drag = null
    if (!d || d.moved) return
    const pt = localPoint(ev)
    const now = performance.now()
    const id = d.peer ? d.peer.id : null
    const double = lastTap.id === id && now - lastTap.t < 320
    lastTap = { id, t: double ? 0 : now }
    clearTimeout(pendingTap)

    if (double) {
        if (id) toggleSeed(id)
        else { const nid = addPeer(undefined, pt.x, pt.y); select(nid); changed(); reheat(0.3) }
        return
    }
    if (!id) { select(null); return }
    pendingTap = setTimeout(() => {
        const sel = state.selected
        if (sel && sel !== id && state.peers.has(sel)) {
            const e = state.edges.get(edgeKey(sel, id))
            setEdge(sel, id, (e ? e.w : 0) + 1)
            reheat(0.25)
        } else select(id)
    }, 220)
})

document.addEventListener('keydown', (ev) => {
    if (ev.target.closest('input, textarea, select, [role="listbox"]')) return
    if (ev.key === 'Escape') select(null)
    if ((ev.key === 'Delete' || ev.key === 'Backspace') && state.selected) removePeer(state.selected)
    if (ev.key === 's' && state.selected) toggleSeed(state.selected)
})

function select(id) {
    state.selected = id
    renderPanel()
    wake()
}

// ---------- panel ----------

const fmtShare = (s) => {
    if (s === 0) return num(0, { style: 'percent' })
    if (s < 0.0001) return '<' + num(0.0001, { style: 'percent', maximumFractionDigits: 2 })
    const digits = s < 0.01 ? 2 : 1
    return num(s, { style: 'percent', minimumFractionDigits: digits, maximumFractionDigits: digits })
}

function el(tag, attrs = {}, ...children) {
    const node = document.createElement(tag)
    for (const [k, v] of Object.entries(attrs)) {
        if (k === 'class') node.className = v
        else if (k.startsWith('on')) node.addEventListener(k.slice(2), v)
        else if (v !== undefined && v !== false) node.setAttribute(k, v === true ? '' : v)
    }
    for (const c of children.flat()) if (c !== null && c !== undefined) node.append(c)
    return node
}

function renderPanel() {
    renderRanking()
    renderSelected()
    renderStage()
}

function renderStage() {
    const note = $('stageNote')
    let text = ''
    if (state.large) {
        text = t('noteLarge', { n: num(state.large.peers) })
    } else if (state.peers.size === 0) {
        text = t('noteEmpty')
    } else if (state.edges.size === 0) {
        text = t('noteNoLinks')
    }
    note.textContent = text
    note.hidden = !text
    for (const b of ['addPeer', 'relayout', 'share']) $(b).disabled = !!state.large
}

function renderRanking() {
    const list = $('ranking')
    list.replaceChildren()
    const stats = $('stats')

    let rows = state.scores
    if (!state.large) {
        const scored = new Set(rows.map(([id]) => id))
        rows = rows.concat([...state.peers.keys()].filter((id) => !scored.has(id)).map((id) => [id, 0]))
    }
    const max = Math.max(1e-12, ...rows.slice(0, 1).map(([, s]) => s))
    const shown = rows.slice(0, 60)
    shown.forEach(([id, score], i) => {
        const button = el('button', {
            type: 'button',
            class: id === state.selected ? 'is-selected' : '',
            onclick: () => { if (!state.large) select(id === state.selected ? null : id) },
            'aria-pressed': String(id === state.selected),
        },
        el('span', { class: 'rank' }, score > 0 ? num(i + 1) : ''),
        el('span', { class: 'name' + (state.seeds.has(id) ? ' is-seed' : ''), title: id }, id),
        el('span', { class: 'bar' }, el('i', { style: `width:${(score / max) * 100}%` })),
        el('span', { class: 'score' }, fmtShare(score)))
        list.append(el('li', {}, button))
    })
    if (rows.length > shown.length) list.append(el('li', { class: 'ranking-more' }, t('more', { n: num(rows.length - shown.length) })))
    if (!rows.length) list.append(el('li', { class: 'ranking-more' }, t('noScores')))

    if (state.large) {
        stats.textContent = t('statPeers', { n: num(state.large.peers) })
    } else {
        const parts = [t('statNetwork', { p: num(state.peers.size), l: num(state.edges.size) })]
        const sybils = [...state.peers.values()].filter((p) => p.group === 'sybil')
        if (sybils.length && state.scores.length) {
            parts.push(t('statSybil', { x: fmtShare(sybils.reduce((s, p) => s + p.score, 0)) }))
        } else if (state.seeds.size === 0 && state.edges.size) {
            parts.push(t('statNoSeeds'))
        }
        stats.textContent = parts.join(lang() === 'zh' || lang() === 'ja' ? '，' : lang() === 'ar' ? '، ' : ', ')
    }
    $('engineError').textContent = state.error || ''
    $('engineError').hidden = !state.error
    $('engine').textContent = engineText()
}

function renderSelected() {
    const box = $('selected')
    const id = state.selected
    if (!id || !state.peers.has(id) || state.large) { box.hidden = true; box.replaceChildren(); return }
    const p = state.peers.get(id)
    const rank = state.scores.findIndex(([pid]) => pid === id)
    const out = [...state.edges.values()].filter((e) => e.from === id)
    const inc = [...state.edges.values()].filter((e) => e.to === id)
    const others = [...state.peers.keys()].filter((o) => o !== id && !state.edges.has(edgeKey(id, o)))
    const isSeed = state.seeds.has(id)

    const addSelect = el('select', { 'aria-label': t('ariaShouldTrust', { id }) }, others.map((o) => el('option', { value: o }, o)))

    box.replaceChildren(
        el('div', { class: 'sel-head' },
            el('h3', {}, id),
            el('span', { class: 'sel-score' }, p.score > 0 ? t('selScore', { x: fmtShare(p.score), r: num(rank + 1) }) : t('noTrustYet'))),
        el('div', { class: 'sel-actions' },
            el('button', { type: 'button', class: isSeed ? 'seed-on' : '', 'aria-pressed': String(isSeed), onclick: () => toggleSeed(id) }, isSeed ? t('isSeed') : t('makeSeed')),
            el('button', { type: 'button', onclick: () => removePeer(id) }, t('removePeer')),
            el('button', { type: 'button', onclick: () => select(null) }, t('done'))),
        el('h4', {}, t('trusts', { n: num(out.length) })),
        out.length
            ? el('ul', { class: 'edges' }, out.map((e) => el('li', {},
                el('span', {}, e.to),
                el('input', {
                    type: 'number', min: '0', step: '1', value: String(e.w), 'aria-label': t('ariaTrust', { a: id, b: e.to }),
                    onchange: (ev) => setEdge(id, e.to, parseFloat(ev.target.value)),
                }),
                el('button', { type: 'button', 'aria-label': t('ariaRemoveTrust', { a: id, b: e.to }), onclick: () => setEdge(id, e.to, 0) }, t('remove')))))
            : el('p', { class: 'empty-note' }, t('trustsEmpty')),
        others.length ? el('div', { class: 'add-edge' }, addSelect,
            el('button', { type: 'button', onclick: () => { setEdge(id, addSelect.value, 1); reheat(0.25) } }, t('addTrust'))) : null,
        el('h4', {}, t('trustedBy', { n: num(inc.length) })),
        el('p', { class: 'empty-note' }, inc.length ? inc.map((e) => `${e.from} (${num(e.w)})`).join(', ') : t('nobody')),
    )
    box.hidden = false
}

// ---------- controls ----------

function setAlpha(a) {
    state.alpha = a
    $('alpha').value = a
    $('alphaOut').textContent = num(a, { minimumFractionDigits: 2, maximumFractionDigits: 2 })
}

$('alpha').addEventListener('input', (ev) => { setAlpha(parseFloat(ev.target.value)); changed() })
$('preset').addEventListener('change', (ev) => loadNetwork(PRESETS[ev.target.value]))
$('addPeer').addEventListener('click', () => { const id = addPeer(); select(id); changed(); reheat(0.4) })
$('relayout').addEventListener('click', () => {
    const { w, h } = size()
    for (const p of state.peers.values()) { p.x = w / 2 + (Math.random() - 0.5) * w * 0.5; p.y = h * 0.56 + (Math.random() - 0.5) * h * 0.5 }
    reheat(1)
})
$('share').addEventListener('click', async () => {
    writeHash()
    try {
        await navigator.clipboard.writeText(location.href)
        $('share').textContent = t('linkCopied')
    } catch {
        $('share').textContent = t('copyFallback')
    }
    setTimeout(() => { $('share').textContent = t('copyLink') }, 1600)
})

// tabs
const tabs = ['network', 'csv', 'bench']
for (const t of tabs) {
    $('tab-' + t).addEventListener('click', () => showTab(t))
}
$('tab-network').parentElement.addEventListener('keydown', (ev) => {
    if (ev.key !== 'ArrowRight' && ev.key !== 'ArrowLeft') return
    const cur = tabs.findIndex((t) => $('tab-' + t).getAttribute('aria-selected') === 'true')
    const forward = (ev.key === 'ArrowRight') !== (document.documentElement.dir === 'rtl')
    const next = tabs[(cur + (forward ? 1 : tabs.length - 1)) % tabs.length]
    showTab(next)
    $('tab-' + next).focus()
})

function showTab(name) {
    for (const t of tabs) {
        const on = t === name
        $('tab-' + t).setAttribute('aria-selected', String(on))
        $('tab-' + t).tabIndex = on ? 0 : -1
        $('view-' + t).hidden = !on
    }
    if (name === 'csv') fillCsv()
}

// ---------- CSV ----------

const bigFiles = { lt: null, pt: null }
const BIG_TEXT = 2_000_000

function fillCsv() {
    if (state.large || bigFiles.lt) return
    const { lt, pt } = networkCsv()
    $('ltText').value = lt ? 'from,to,weight\n' + lt : ''
    $('ptText').value = pt ? 'peer,weight\n' + pt : ''
}

for (const [key, fileId, textId] of [['lt', 'ltFile', 'ltText'], ['pt', 'ptFile', 'ptText']]) {
    $(fileId).addEventListener('change', async (ev) => {
        const file = ev.target.files[0]
        if (!file) return
        const text = await file.text()
        if (text.length > BIG_TEXT) {
            bigFiles[key] = text
            $(textId).value = t('tooLarge', { name: file.name, n: num(countLines(text)) })
            $(textId).readOnly = true
        } else {
            bigFiles[key] = null
            $(textId).readOnly = false
            $(textId).value = text
        }
        ev.target.value = ''
    })
    $(textId).addEventListener('focus', () => {
        if (bigFiles[key]) return
        $(textId).readOnly = false
    })
}

function countLines(s) {
    let n = 0
    for (let i = 0; i < s.length; i++) if (s.charCodeAt(i) === 10) n++
    return n + (s.length && s[s.length - 1] !== '\n' ? 1 : 0)
}

const HEADER_NAMES = new Set(['i', 'j', 'v', 'from', 'to', 'value', 'weight', 'trust', 'level', 'peer', 'id', 'score',
    'source', 'target', 'src', 'dst', 'truster', 'trustee'])
// one CSV line: quoted fields may contain commas and doubled quotes, same rules as the engine
function splitCsvLine(line) {
    const out = []
    let cur = ''
    let quoted = false
    for (let i = 0; i < line.length; i++) {
        const c = line[i]
        if (quoted) {
            if (c !== '"') cur += c
            else if (line[i + 1] === '"') { cur += '"'; i++ }
            else quoted = false
        } else if (c === '"' && !cur.trim()) { cur = ''; quoted = true }
        else if (c === ',') { out.push(cur.trim()); cur = '' }
        else cur += c
    }
    out.push(cur.trim())
    return out
}

function parseRows(text, valueCol) {
    const rows = []
    let first = true
    for (const line of text.replace(/^\uFEFF/, '').split(/\r?\n/)) {
        if (!line.trim()) continue
        const f = splitCsvLine(line)
        if (first) {
            first = false
            const header = f.length > valueCol ? isNaN(Number(f[valueCol])) : f.every((x) => HEADER_NAMES.has(x.toLowerCase()))
            if (header) continue
        }
        rows.push(f)
    }
    return rows
}

// quote a field when it needs it
const csvField = (v) => /[",\r\n]|^\s|\s$/.test(String(v)) ? '"' + String(v).replace(/"/g, '""') + '"' : String(v)

$('csvLoad').addEventListener('click', async () => {
    const err = $('csvError')
    err.hidden = true
    const lt = bigFiles.lt ?? $('ltText').value
    const pt = bigFiles.pt ?? $('ptText').value
    const btn = $('csvLoad')
    btn.disabled = true
    btn.textContent = t('busy')
    const res = await runEngine(lt, pt, state.alpha)
    btn.disabled = false
    btn.textContent = t('loadNetwork')
    if (res.error) {
        err.textContent = t('rejected', { e: res.error })
        err.hidden = false
        return
    }
    const peers = new Set()
    const ltRows = lt.length > BIG_TEXT ? null : parseRows(lt, 2)
    if (ltRows) for (const r of ltRows) { peers.add(r[0]); peers.add(r[1]) }
    if (!ltRows || peers.size > MAX_GRAPH_PEERS) {
        clearNetwork()
        const count = ltRows ? peers.size : new Set(lt.split('\n').flatMap((l) => splitCsvLine(l).slice(0, 2)).filter(Boolean)).size
        state.large = { lt, pt, peers: count }
        state.scores = res.scores
        state.lastRun = { ms: res.ms, threads: res.threads }
        applyScores()
        renderPanel()
        wake()
        return
    }
    const edges = ltRows.map((r) => [r[0], r[1], r.length > 2 ? parseFloat(r[2]) : 1])
    const seeds = parseRows(pt, 1).map((r) => r[0])
    loadNetwork({ edges, seeds })
})

$('csvDownload').addEventListener('click', () => {
    const csv = 'peer,score\n' + state.scores.map(([p, s]) => `${csvField(p)},${s}`).join('\n') + '\n'
    const a = el('a', { href: URL.createObjectURL(new Blob([csv], { type: 'text/csv' })), download: 'eigentrust-scores.csv' })
    a.click()
    setTimeout(() => URL.revokeObjectURL(a.href), 1000)
})

// ---------- benchmark ----------

function generateNetwork(n, degree) {
    // half uniform links, half skewed toward low ids so some peers become hubs
    const enc = new TextEncoder()
    const parts = []
    let chunk = ''
    for (let i = 0; i < n; i++) {
        for (let k = 0; k < degree; k++) {
            const j = Math.random() < 0.5 ? Math.floor(Math.random() * n) : Math.floor(n * Math.random() ** 3)
            if (j === i) continue
            chunk += `p${i},p${j},${1 + Math.floor(Math.random() * 10)}\n`
        }
        if (chunk.length > 1 << 20) { parts.push(enc.encode(chunk)); chunk = '' }
    }
    parts.push(enc.encode(chunk))
    const total = parts.reduce((s, p) => s + p.length, 0)
    const bytes = new Uint8Array(total)
    let off = 0
    for (const p of parts) { bytes.set(p, off); off += p.length }
    let pt = ''
    for (let i = 0; i < Math.min(20, n); i++) pt += `p${i},1\n`
    return { lt: bytes, pt: enc.encode(pt), edges: countNewlines(bytes) }
}

function countNewlines(bytes) {
    let n = 0
    for (let i = 0; i < bytes.length; i++) if (bytes[i] === 10) n++
    return n
}

$('benchRun').addEventListener('click', async () => {
    const n = parseInt($('benchPeers').value, 10)
    const degree = parseInt($('benchDegree').value, 10)
    const out = $('benchOut')
    const btn = $('benchRun')
    btn.disabled = true
    out.replaceChildren(el('p', {}, t('generating', { n: num(n) })))
    await new Promise((r) => setTimeout(r, 30))
    const t0 = performance.now()
    const net = generateNetwork(n, degree)
    const genMs = performance.now() - t0
    out.replaceChildren(el('p', {}, t('rankingN', { n: num(net.edges) })))
    const size = net.lt.length
    const res = await runEngine(net.lt, net.pt, state.alpha)
    btn.disabled = false
    if (res.error) {
        out.replaceChildren(el('p', { class: 'error' }, res.error))
        return
    }
    out.replaceChildren(
        el('div', { class: 'big' }, `${num(Math.round(res.ms))} ms`),
        el('p', {}, t('benchResult', {
            mb: num(size / 1e6, { maximumFractionDigits: 1 }), n: num(n), e: num(net.edges),
            a: num(state.alpha, { minimumFractionDigits: 2 }), mode: modeText(res.threads), g: num(Math.round(genMs)),
        })),
        el('ol', { class: 'ranking' }, res.scores.slice(0, 8).map(([p, s], i) => el('li', {}, el('button', { type: 'button', tabindex: '-1' },
            el('span', { class: 'rank' }, String(i + 1)), el('span', { class: 'name' }, p),
            el('span', { class: 'bar' }, el('i', { style: `width:${(s / res.scores[0][1]) * 100}%` })),
            el('span', { class: 'score' }, fmtShare(s)))))),
    )
})

// ---------- share links ----------

let hashTimer = 0
function writeHash() {
    clearTimeout(hashTimer)
    hashTimer = setTimeout(() => {
        if (state.large) return
        const ids = [...state.peers.keys()]
        const index = new Map(ids.map((id, i) => [id, i]))
        const data = {
            p: ids,
            e: [...state.edges.values()].map((e) => [index.get(e.from), index.get(e.to), e.w]),
            s: [...state.seeds].map((s) => index.get(s)),
            a: state.alpha,
            g: ids.map((id) => state.peers.get(id).group ? 1 : 0),
        }
        const json = JSON.stringify(data)
        if (json.length > 6000) { history.replaceState(null, '', location.pathname); return }
        const b64 = btoa(unescape(encodeURIComponent(json))).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
        history.replaceState(null, '', '#n=' + b64)
    }, 300)
}

function readHash() {
    const m = location.hash.match(/^#n=([\w-]+)$/)
    if (!m) return false
    try {
        const json = decodeURIComponent(escape(atob(m[1].replace(/-/g, '+').replace(/_/g, '/'))))
        const d = JSON.parse(json)
        const edges = d.e.map(([i, j, w]) => [d.p[i], d.p[j], w])
        const groups = { sybil: d.p.filter((_, i) => d.g?.[i]) }
        loadNetwork({ edges, seeds: d.s.map((i) => d.p[i]), groups, alpha: d.a })
        for (const id of d.p) if (!state.peers.has(id)) addPeer(id)
        return true
    } catch {
        return false
    }
}

// ---------- language ----------

function localizeNumbers() {
    for (const o of $('benchPeers').options) o.textContent = num(parseInt(o.value, 10))
    for (const o of $('benchDegree').options) o.textContent = num(parseInt(o.value, 10))
}

function applyLanguage(code) {
    setLanguage(code)
    renderLangMenu()
    localizeNumbers()
    setAlpha(state.alpha)
    renderPanel()
    wake()
}

// language menu: a button that opens a listbox, keyboard and pointer friendly
const langButton = $('langButton')
const langList = $('langList')
const langCodes = Object.keys(LANGUAGES)
let langActive = 0

function renderLangMenu() {
    const cur = lang()
    $('langCurrent').textContent = LANGUAGES[cur]
    $('langCode').textContent = cur.toUpperCase()
    langButton.setAttribute('aria-label', `${t('language')}: ${LANGUAGES[cur]}`)
    langList.setAttribute('aria-label', t('language'))
    langList.replaceChildren(...langCodes.map((code, i) => el('li', {
        id: 'lang-opt-' + code,
        role: 'option',
        'aria-selected': String(code === cur),
        onclick: () => chooseLang(code),
        onpointermove: () => setLangActive(i),
    }, el('span', { lang: code, dir: 'auto' }, LANGUAGES[code]), code === cur ? checkIcon() : null)))
}

function checkIcon() {
    const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg')
    svg.setAttribute('viewBox', '0 0 12 12')
    svg.setAttribute('width', '12')
    svg.setAttribute('height', '12')
    svg.setAttribute('aria-hidden', 'true')
    svg.classList.add('check')
    svg.innerHTML = '<path d="M2 6.5 4.8 9 10 3"/>'
    return svg
}

function setLangActive(i) {
    langActive = (i + langCodes.length) % langCodes.length
    langList.querySelectorAll('li').forEach((li, j) => li.classList.toggle('is-active', j === langActive))
    langList.setAttribute('aria-activedescendant', 'lang-opt-' + langCodes[langActive])
}

function openLangMenu() {
    langList.hidden = false
    langButton.setAttribute('aria-expanded', 'true')
    setLangActive(langCodes.indexOf(lang()))
    langList.focus()
}

function closeLangMenu(focusButton = true) {
    if (langList.hidden) return
    langList.hidden = true
    langButton.setAttribute('aria-expanded', 'false')
    if (focusButton) langButton.focus()
}

function chooseLang(code) {
    closeLangMenu()
    if (code === lang()) return
    saveLanguage(code)
    applyLanguage(code)
}

langButton.addEventListener('click', () => (langList.hidden ? openLangMenu() : closeLangMenu()))
langButton.addEventListener('keydown', (ev) => {
    if (ev.key === 'ArrowDown' || ev.key === 'ArrowUp') { ev.preventDefault(); openLangMenu() }
})
langList.addEventListener('keydown', (ev) => {
    const keys = { ArrowDown: 1, ArrowUp: -1 }
    if (ev.key in keys) setLangActive(langActive + keys[ev.key])
    else if (ev.key === 'Home') setLangActive(0)
    else if (ev.key === 'End') setLangActive(langCodes.length - 1)
    else if (ev.key === 'Enter' || ev.key === ' ') chooseLang(langCodes[langActive])
    else if (ev.key === 'Escape') closeLangMenu()
    else if (ev.key === 'Tab') { closeLangMenu(false); return }
    else return
    ev.preventDefault()
})
document.addEventListener('pointerdown', (ev) => {
    if (!langList.hidden && !$('langMenu').contains(ev.target)) closeLangMenu(false)
})

// ---------- start ----------

applyLanguage(detectLanguage())
resize()
if (!readHash()) loadNetwork(PRESETS.friends)
