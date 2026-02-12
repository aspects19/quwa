import { FileText, Database } from 'lucide-react';

interface Source {
  source_type: string;
  source_id: string;
  relevance: number;
}

interface SourceCitationProps {
  sources: Source[];
}

export default function SourceCitation({ sources }: SourceCitationProps) {
  if (!sources || sources.length === 0) return null;

  const getSourceIcon = (type: string) => {
    if (type === 'user_file') return FileText;
    if (type === 'orphadata') return Database;
    return Database;
  };

  const getSourceLabel = (type: string) => {
    if (type === 'user_file') return 'Patient File';
    if (type === 'orphadata') return 'Orphadata';
    return 'Source';
  };

  const formatRelevance = (score: number) => {
    return `${Math.round(score * 100)}%`;
  };

  return (
    <div className="mt-4 space-y-2">
      <p className="text-xs font-medium text-white/50 uppercase tracking-wide">
        Sources ({sources.length})
      </p>
      <div className="space-y-2">
        {sources.map((source, idx) => {
          const Icon = getSourceIcon(source.source_type);
          const label = getSourceLabel(source.source_type);

          return (
            <div
              key={`${source.source_id}-${idx}`}
              className="flex items-center gap-3 p-3 rounded-lg bg-white/5 border border-white/10"
            >
              <Icon className="w-4 h-4 text-white/60 shrink-0" />
              
              <div className="flex-1 min-w-0">
                <p className="text-sm text-white/80">{label}</p>
                <p className="text-xs text-white/50 truncate">
                  {source.source_id}
                </p>
              </div>

              <div className="flex items-center gap-2 shrink-0">
                <div className="text-xs text-white/60">
                  {formatRelevance(source.relevance)}
                </div>
                <div className="w-12 h-1.5 bg-white/10 rounded-full overflow-hidden">
                  <div
                    className="h-full bg-primary rounded-full transition-all"
                    style={{ width: `${source.relevance * 100}%` }}
                  />
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
