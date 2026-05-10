/**
 * Language / Repository Breakdown Card
 *
 * Renders the language distribution (pie + legend) and the repository-wise
 * additions/deletions chart introduced in Issue #193 (audit §1 G-11).
 *
 * Both visualizations share a single GraphQL call (24h cache) — see
 * `github.getLanguageBreakdownWithCache` and `get_language_breakdown` on
 * the backend. The component is purely presentational: it expects a
 * resolved payload and renders skeletons / empty states locally.
 *
 * Charts are SVG-based on purpose: this app has no chart library
 * dependency and the dataset is small (≤ 10 languages, ≤ 10 repos shown)
 * so handrolled SVG keeps the bundle lean.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/193
 */

import React, { useMemo } from 'react';
import type {
  LanguageBreakdownResponse,
  LanguageStats,
  RepositoryCodeStats,
} from '../../../types';

interface LanguageBreakdownCardProps {
  data: LanguageBreakdownResponse | null;
  isLoading: boolean;
  error: string | null;
  fromCache?: boolean;
}

const MAX_REPOS_SHOWN = 8;
const MAX_LANGUAGES_LEGEND = 8;

/// Fallback palette for languages whose `color` field is null. Chosen to
/// stay readable on the dark dashboard background and rotated by index so
/// adjacent slices remain distinguishable.
const FALLBACK_COLORS = [
  '#22d3ee',
  '#a78bfa',
  '#f472b6',
  '#34d399',
  '#fbbf24',
  '#fb7185',
  '#60a5fa',
  '#f97316',
  '#84cc16',
  '#e879f9',
];

function languageColor(language: LanguageStats, fallbackIndex: number): string {
  return language.color ?? FALLBACK_COLORS[fallbackIndex % FALLBACK_COLORS.length];
}

function formatPercent(value: number): string {
  if (value <= 0) return '0%';
  if (value < 0.001) return '<0.1%';
  return `${(value * 100).toFixed(1)}%`;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

function formatLines(value: number): string {
  if (value < 1000) return value.toString();
  if (value < 1_000_000) return `${(value / 1000).toFixed(1)}k`;
  return `${(value / 1_000_000).toFixed(2)}M`;
}

interface PieSlice {
  startAngle: number;
  endAngle: number;
  language: LanguageStats;
  fallbackIndex: number;
}

/// Build the slice geometry. Languages with sub-percent share are folded
/// into "Other" so the SVG path stays meaningful at small radii.
function buildPieData(languages: LanguageStats[]): {
  slices: PieSlice[];
  legend: Array<{ language: LanguageStats; fallbackIndex: number }>;
} {
  if (languages.length === 0) {
    return { slices: [], legend: [] };
  }

  const total = languages.reduce((sum, l) => sum + l.bytes, 0);
  if (total <= 0) {
    return { slices: [], legend: [] };
  }

  const visible = languages.slice(0, MAX_LANGUAGES_LEGEND);
  const restBytes = languages
    .slice(MAX_LANGUAGES_LEGEND)
    .reduce((sum, l) => sum + l.bytes, 0);

  const entries: LanguageStats[] = restBytes > 0
    ? [
        ...visible,
        {
          name: 'Other',
          color: '#475569',
          bytes: restBytes,
          percentage: restBytes / total,
        },
      ]
    : visible;

  let cumulative = 0;
  const slices: PieSlice[] = entries.map((language, idx) => {
    const angle = (language.bytes / total) * Math.PI * 2;
    const slice: PieSlice = {
      startAngle: cumulative,
      endAngle: cumulative + angle,
      language,
      fallbackIndex: idx,
    };
    cumulative += angle;
    return slice;
  });

  return {
    slices,
    legend: entries.map((language, idx) => ({ language, fallbackIndex: idx })),
  };
}

function pieSlicePath(
  slice: PieSlice,
  cx: number,
  cy: number,
  radius: number,
): string {
  const startX = cx + radius * Math.cos(slice.startAngle - Math.PI / 2);
  const startY = cy + radius * Math.sin(slice.startAngle - Math.PI / 2);
  const endX = cx + radius * Math.cos(slice.endAngle - Math.PI / 2);
  const endY = cy + radius * Math.sin(slice.endAngle - Math.PI / 2);

  const sweep = slice.endAngle - slice.startAngle;
  // A full-circle slice (single dominant language) needs to be drawn as a
  // closed circle path because the start/end points coincide and the SVG
  // arc command would otherwise render nothing.
  if (sweep >= Math.PI * 2 - 1e-6) {
    return `M ${cx - radius} ${cy} a ${radius} ${radius} 0 1 0 ${radius * 2} 0 a ${radius} ${radius} 0 1 0 ${-radius * 2} 0 Z`;
  }

  const largeArc = sweep > Math.PI ? 1 : 0;
  return [
    `M ${cx} ${cy}`,
    `L ${startX} ${startY}`,
    `A ${radius} ${radius} 0 ${largeArc} 1 ${endX} ${endY}`,
    'Z',
  ].join(' ');
}

const SkeletonView: React.FC = () => (
  <div className="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-cyan/20 animate-pulse">
    <div className="h-6 w-48 bg-slate-700 rounded mb-4"></div>
    <div className="grid grid-cols-1 lg:grid-cols-[14rem_1fr] gap-6">
      <div className="h-56 bg-slate-700 rounded-xl"></div>
      <div className="space-y-3">
        {Array.from({ length: 6 }).map((_, i) => (
          <div key={i} className="h-8 bg-slate-700 rounded-lg"></div>
        ))}
      </div>
    </div>
  </div>
);

const ErrorView: React.FC<{ message: string }> = ({ message }) => (
  <div className="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-red-500/30">
    <h3 className="text-lg font-semibold text-red-300 mb-2">
      言語別 / リポジトリ別統計を読み込めませんでした
    </h3>
    <p className="text-sm text-dt-text-sub">{message}</p>
  </div>
);

const EmptyView: React.FC = () => (
  <div className="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-cyan/20">
    <h3 className="text-lg font-semibold text-dt-text-main mb-2">
      言語別 / リポジトリ別統計
    </h3>
    <p className="text-sm text-dt-text-sub">
      集計対象のリポジトリが見つかりませんでした。GitHub にコミットを公開すると
      ここに表示されます。
    </p>
  </div>
);

const LanguagePie: React.FC<{ data: LanguageBreakdownResponse }> = ({ data }) => {
  const { slices, legend } = useMemo(() => buildPieData(data.languages), [data.languages]);

  if (slices.length === 0) {
    return (
      <div className="flex items-center justify-center h-56 text-sm text-dt-text-sub">
        言語データがありません
      </div>
    );
  }

  const SIZE = 200;
  const RADIUS = 90;
  const CX = SIZE / 2;
  const CY = SIZE / 2;

  return (
    <div className="flex flex-col items-center gap-4">
      <svg width={SIZE} height={SIZE} viewBox={`0 0 ${SIZE} ${SIZE}`} role="img" aria-label="言語別バイト比率">
        {slices.map((slice) => (
          <path
            key={slice.language.name}
            d={pieSlicePath(slice, CX, CY, RADIUS)}
            fill={languageColor(slice.language, slice.fallbackIndex)}
            stroke="#0f172a"
            strokeWidth={1}
          >
            <title>
              {slice.language.name} — {formatPercent(slice.language.percentage)} ({formatBytes(slice.language.bytes)})
            </title>
          </path>
        ))}
        <circle cx={CX} cy={CY} r={RADIUS * 0.55} fill="#0f172a" />
        <text
          x={CX}
          y={CY - 6}
          textAnchor="middle"
          className="fill-dt-text-main"
          style={{ fontSize: '12px' }}
        >
          {data.repositoriesScanned} repos
        </text>
        <text
          x={CX}
          y={CY + 12}
          textAnchor="middle"
          className="fill-dt-text-sub"
          style={{ fontSize: '11px' }}
        >
          {formatBytes(data.totalBytes)}
        </text>
      </svg>

      <ul className="w-full space-y-1.5 text-sm">
        {legend.map(({ language, fallbackIndex }) => (
          <li
            key={language.name}
            className="flex items-center justify-between gap-2"
          >
            <span className="flex items-center gap-2 min-w-0">
              <span
                className="inline-block w-3 h-3 rounded-sm flex-shrink-0"
                style={{ backgroundColor: languageColor(language, fallbackIndex) }}
                aria-hidden="true"
              />
              <span className="truncate text-dt-text-main">{language.name}</span>
            </span>
            <span className="text-dt-text-sub font-gaming-mono text-xs flex-shrink-0">
              {formatPercent(language.percentage)}
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
};

const RepositoryBars: React.FC<{ repositories: RepositoryCodeStats[] }> = ({ repositories }) => {
  const visible = repositories.slice(0, MAX_REPOS_SHOWN);
  const peakChurn = useMemo(() => {
    return visible.reduce((max, r) => Math.max(max, r.additions + r.deletions), 0);
  }, [visible]);

  if (visible.length === 0) {
    return (
      <div className="flex items-center justify-center h-32 text-sm text-dt-text-sub">
        過去 90 日のコミット履歴に該当するリポジトリがありません
      </div>
    );
  }

  return (
    <div className="space-y-3">
      <div className="flex items-baseline justify-between text-xs text-dt-text-sub">
        <span>過去 90 日 — 上位 {visible.length} リポジトリ</span>
        <span className="font-gaming-mono">+追加 / −削除</span>
      </div>

      <ul className="space-y-3">
        {visible.map((repo) => {
          const churn = repo.additions + repo.deletions;
          const additionsRatio = peakChurn > 0 ? repo.additions / peakChurn : 0;
          const deletionsRatio = peakChurn > 0 ? repo.deletions / peakChurn : 0;

          return (
            <li key={repo.nameWithOwner} className="space-y-1">
              <div className="flex items-center justify-between gap-2 text-sm">
                <span className="flex items-center gap-2 min-w-0">
                  {repo.primaryLanguage && (
                    <span
                      className="inline-block w-2.5 h-2.5 rounded-full flex-shrink-0"
                      style={{
                        backgroundColor:
                          repo.primaryLanguageColor ?? FALLBACK_COLORS[0],
                      }}
                      title={repo.primaryLanguage}
                      aria-label={`Primary language: ${repo.primaryLanguage}`}
                    />
                  )}
                  {repo.url ? (
                    <a
                      href={repo.url}
                      target="_blank"
                      rel="noreferrer noopener"
                      className="truncate text-dt-text-main hover:text-gm-accent-cyan transition-colors"
                      title={repo.nameWithOwner}
                    >
                      {repo.nameWithOwner}
                    </a>
                  ) : (
                    <span className="truncate text-dt-text-main" title={repo.nameWithOwner}>
                      {repo.nameWithOwner}
                    </span>
                  )}
                </span>
                <span className="text-xs text-dt-text-sub font-gaming-mono whitespace-nowrap">
                  <span className="text-emerald-400">+{formatLines(repo.additions)}</span>
                  <span className="mx-1 text-slate-500">·</span>
                  <span className="text-rose-400">−{formatLines(repo.deletions)}</span>
                  <span className="mx-1 text-slate-500">·</span>
                  <span>{repo.commitsCount} commits</span>
                </span>
              </div>

              <div className="flex h-2 rounded-full overflow-hidden bg-slate-800/60">
                <div
                  className="bg-emerald-500/80"
                  style={{ width: `${additionsRatio * 100}%` }}
                  aria-hidden="true"
                />
                <div
                  className="bg-rose-500/80"
                  style={{ width: `${deletionsRatio * 100}%` }}
                  aria-hidden="true"
                />
              </div>

              <span className="sr-only">
                {repo.nameWithOwner} added {repo.additions} lines and removed {repo.deletions}
                lines across {repo.commitsCount} commits ({formatLines(churn)} total churn).
              </span>
            </li>
          );
        })}
      </ul>
    </div>
  );
};

export const LanguageBreakdownCard: React.FC<LanguageBreakdownCardProps> = ({
  data,
  isLoading,
  error,
  fromCache,
}) => {
  if (isLoading && data === null && error === null) {
    return <SkeletonView />;
  }

  if (error !== null && data === null) {
    return <ErrorView message={error} />;
  }

  if (data === null) {
    return <SkeletonView />;
  }

  const isEmpty =
    data.languages.length === 0 && data.repositories.length === 0;

  if (isEmpty) {
    return <EmptyView />;
  }

  return (
    <div className="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-cyan/20">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-dt-text-main">
          言語別 / リポジトリ別統計
        </h3>
        {fromCache ? (
          <span
            className="text-xs text-dt-text-sub px-2 py-0.5 rounded-full bg-slate-800/80"
            title="キャッシュから表示中（バックグラウンドで更新）"
          >
            キャッシュ
          </span>
        ) : null}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-[14rem_1fr] gap-6">
        <LanguagePie data={data} />
        <RepositoryBars repositories={data.repositories} />
      </div>
    </div>
  );
};
