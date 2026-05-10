const { invoke } = window.__TAURI__.core;
const { listen }  = window.__TAURI__.event;

// ── State ─────────────────────────────────────────────────────────────────────

let channels        = [];
let layoutView      = 'list';
let currentUsername = '';
let sortCol         = 'date';
let sortDir         = 'desc';

// ── Element refs ──────────────────────────────────────────────────────────────

const viewEnter     = document.getElementById('view-enter');
const viewList      = document.getElementById('view-list');
const usernameInput = document.getElementById('username-input');
const loadingPanel  = document.getElementById('loading');
const loadingTitle  = document.getElementById('loading-title');
const progressBar   = document.getElementById('progress-bar');
const loadingHint   = document.getElementById('loading-hint');
const errorPanel    = document.getElementById('error-panel');
const tableWrap     = document.getElementById('table-wrap');
const tableTitle    = document.getElementById('table-title');
const channelRows   = document.getElementById('channel-rows');
const listContainer = document.getElementById('list-container');
const gridContainer = document.getElementById('grid-container');
const gridSortBar   = document.getElementById('grid-sort-bar');
const btnList       = document.getElementById('btn-list');
const btnGrid       = document.getElementById('btn-grid');

// ── View switching ─────────────────────────────────────────────────────────────

function showEnterView() {
  viewList.classList.add('hidden');
  viewEnter.classList.remove('hidden');
  usernameInput.focus();
  usernameInput.select();
}

function showResultView() {
  viewEnter.classList.add('hidden');
  viewList.classList.remove('hidden');
}

// ── Layout toggle ──────────────────────────────────────────────────────────────

function setLayout(view) {
  layoutView = view;
  listContainer.classList.toggle('hidden', view !== 'list');
  gridContainer.classList.toggle('hidden', view !== 'grid');
  gridSortBar.classList.toggle('hidden', view !== 'grid');
  btnList.classList.toggle('active', view === 'list');
  btnGrid.classList.toggle('active', view === 'grid');
  if (view === 'grid' && channels.length > 0) {
    renderGrid();
    updateGridSortBar();
  }
}

btnList.addEventListener('click', () => setLayout('list'));
btnGrid.addEventListener('click', () => setLayout('grid'));
document.getElementById('btn-back').addEventListener('click', showEnterView);

// ── Sort ───────────────────────────────────────────────────────────────────────

function sortedChannels() {
  const arr = [...channels];
  arr.sort((a, b) => {
    let cmp = 0;
    switch (sortCol) {
      case 'name': {
        const na = (isAscii(a.displayName) ? a.displayName : a.login).toLowerCase();
        const nb = (isAscii(b.displayName) ? b.displayName : b.login).toLowerCase();
        cmp = na.localeCompare(nb);
        break;
      }
      case 'followers': {
        cmp = (a.follower_count ?? -1) - (b.follower_count ?? -1);
        break;
      }
      case 'date': {
        cmp = (a.followed_at ?? '').localeCompare(b.followed_at ?? '');
        break;
      }
      case 'mutual': {
        cmp = (a.is_mutual ? 1 : 0) - (b.is_mutual ? 1 : 0);
        break;
      }
    }
    return sortDir === 'asc' ? cmp : -cmp;
  });
  return arr;
}

function applySort() {
  renderTable();
  updateSortHeaders();
  renderGrid();
  updateGridSortBar();
}

function updateSortHeaders() {
  document.querySelectorAll('th.sortable').forEach(th => {
    const arrow = th.querySelector('.sort-arrow');
    if (th.dataset.col === sortCol) {
      arrow.textContent = sortDir === 'asc' ? '▲' : '▼';
      th.classList.add('sort-active');
    } else {
      arrow.textContent = '';
      th.classList.remove('sort-active');
    }
  });
}

function updateGridSortBar() {
  const labels = { name: 'Name', followers: 'Followers', date: 'Date', mutual: 'Mutual' };
  document.querySelectorAll('.grid-sort-btn').forEach(btn => {
    const col = btn.dataset.col;
    const active = col === sortCol;
    btn.classList.toggle('active', active);
    btn.textContent = labels[col] + (active ? (sortDir === 'asc' ? ' ▲' : ' ▼') : '');
  });
}

document.querySelectorAll('th.sortable').forEach(th => {
  th.addEventListener('click', () => {
    const col = th.dataset.col;
    if (sortCol === col) {
      sortDir = sortDir === 'asc' ? 'desc' : 'asc';
    } else {
      sortCol = col;
      sortDir = col === 'mutual' ? 'desc' : 'asc';
    }
    applySort();
  });
});

document.querySelectorAll('.grid-sort-btn').forEach(btn => {
  btn.addEventListener('click', () => {
    const col = btn.dataset.col;
    if (sortCol === col) {
      sortDir = sortDir === 'asc' ? 'desc' : 'asc';
    } else {
      sortCol = col;
      sortDir = col === 'mutual' ? 'desc' : 'asc';
    }
    applySort();
  });
});

// ── Search ─────────────────────────────────────────────────────────────────────

async function search(username) {
  username = username.trim();
  if (!username) return;

  showResultView();
  channels = [];

  loadingPanel.classList.remove('hidden');
  tableWrap.classList.add('hidden');
  errorPanel.classList.add('hidden');
  loadingTitle.textContent = `Fetching follows for '${username}'…`;
  progressBar.style.width  = '0%';
  loadingHint.textContent  = 'Fetching follows…';

  let unlistenDetails = () => {};
  let unlistenMutuals = () => {};

  try {
    unlistenDetails = await listen('loading-details', () => {
      progressBar.style.width = '33%';
      loadingHint.textContent = 'Fetching follower counts…';
    });
    unlistenMutuals = await listen('loading-mutuals', () => {
      progressBar.style.width = '66%';
      loadingHint.textContent = 'Checking mutuals…';
    });

    channels = await invoke('fetch_follows', { username });
    progressBar.style.width = '100%';
    loadingHint.textContent = 'Done';

    currentUsername = username;
    applySort();
    setLayout(layoutView);

    loadingPanel.classList.add('hidden');
    tableWrap.classList.remove('hidden');
  } catch (err) {
    loadingPanel.classList.add('hidden');
    errorPanel.textContent = `Error: ${err}`;
    errorPanel.classList.remove('hidden');
    console.error('search error:', err);
  } finally {
    unlistenDetails();
    unlistenMutuals();
  }
}

// ── List rendering ─────────────────────────────────────────────────────────────

const SEARCH_SVG = `<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/></svg>`;

function renderTable() {
  tableTitle.textContent = `${currentUsername}'s Following (${channels.length.toLocaleString()} results)`;
  channelRows.innerHTML  = '';

  for (const ch of sortedChannels()) {
    const name      = isAscii(ch.displayName) ? ch.displayName : ch.login;
    const followers = ch.follower_count != null ? ch.follower_count.toLocaleString() : '—';
    const date      = ch.followed_at ?? '—';
    const avatar    = ch.profileImageURL
      ? `<img class="avatar" src="${ch.profileImageURL}" alt="" loading="lazy">`
      : `<div class="avatar avatar-placeholder"></div>`;

    const tr = document.createElement('tr');
    tr.innerHTML = `
      <td class="name-cell"><div class="name-row">${avatar}<span>${esc(name)}</span></div></td>
      <td class="followers-cell">${followers}</td>
      <td class="date-cell">${esc(date)}</td>
      <td>${ch.is_mutual ? '<span class="mutual-badge">mutual</span>' : ''}</td>
      <td class="search-cell">
        <button class="search-icon-btn" title="Search this user" aria-label="Search this user">${SEARCH_SVG}</button>
      </td>
    `;

    tr.addEventListener('click', () => invoke('open_channel', { login: ch.login }));
    tr.querySelector('.search-icon-btn').addEventListener('click', (e) => {
      e.stopPropagation();
      usernameInput.value = ch.login;
      search(ch.login);
    });

    channelRows.appendChild(tr);
  }
}

// ── Grid rendering ─────────────────────────────────────────────────────────────

function renderGrid() {
  gridContainer.innerHTML = '';

  for (const ch of sortedChannels()) {
    const name = isAscii(ch.displayName) ? ch.displayName : ch.login;

    const card = document.createElement('div');
    card.className = 'channel-card' + (ch.is_mutual ? ' mutual' : '');

    card.innerHTML = ch.profileImageURL
      ? `<img class="card-avatar" src="${ch.profileImageURL}" alt="" loading="lazy">`
      : `<div class="card-avatar card-avatar-placeholder"></div>`;
    card.innerHTML += `<span class="card-name" title="${esc(name)}">${esc(name)}</span>`;
    card.innerHTML += `<button class="card-search-btn" title="Search this user" aria-label="Search this user">${SEARCH_SVG}</button>`;

    card.addEventListener('click', () => invoke('open_channel', { login: ch.login }));
    card.querySelector('.card-search-btn').addEventListener('click', (e) => {
      e.stopPropagation();
      usernameInput.value = ch.login;
      search(ch.login);
    });
    gridContainer.appendChild(card);
  }
}

// ── Search form wiring ─────────────────────────────────────────────────────────

document.getElementById('search-btn').addEventListener('click', () => {
  search(usernameInput.value);
});

usernameInput.addEventListener('keydown', (e) => {
  if (e.key === 'Enter') search(usernameInput.value);
});

// ── Helpers ────────────────────────────────────────────────────────────────────

function esc(str) {
  if (!str) return '';
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
}

function isAscii(str) {
  return /^[\x00-\x7F]*$/.test(str);
}

// ── Init ───────────────────────────────────────────────────────────────────────

usernameInput.focus();
