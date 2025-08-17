import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import type { Problem, Submission } from '@shared/schema';

// Question SDK - Core hooks and utilities for question types
export function useProblem(problemId: string) {
  const { data: problem, isLoading, error } = useQuery<Problem>({
    queryKey: ['/api/problems', problemId],
    enabled: !!problemId,
  });

  return { problem, loading: isLoading, error };
}

export function useSubmitSolution() {
  const queryClient = useQueryClient();

  const mutation = useMutation({
    mutationFn: async (data: { 
      problemId: string; 
      contestId: string; 
      code: string; 
      language: string;
      userId?: string;
    }) => {
      const response = await fetch('/api/submissions', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
      });
      
      if (!response.ok) {
        throw new Error('Failed to submit solution');
      }
      
      return response.json();
    },
    onSuccess: (data, variables) => {
      queryClient.invalidateQueries({ 
        queryKey: ['/api/problems', variables.problemId, 'submissions'] 
      });
    },
  });

  return {
    submitSolution: mutation.mutate,
    submitting: mutation.isPending,
    error: mutation.error,
  };
}

export function useLiveJudgingStream(submissionId: string) {
  const { data: submission, isLoading } = useQuery<Submission>({
    queryKey: ['/api/submissions', submissionId],
    enabled: !!submissionId,
    refetchInterval: 1000, // Poll every second for updates
  });

  return { submission, loading: isLoading };
}

export function useProblemSubmissions(problemId: string, userId?: string) {
  const { data: submissions, isLoading } = useQuery<Submission[]>({
    queryKey: ['/api/problems', problemId, 'submissions'],
    queryFn: async () => {
      const response = await fetch(`/api/problems/${problemId}/submissions?userId=${userId || 'user-1'}`);
      if (!response.ok) {
        throw new Error('Failed to fetch submissions');
      }
      return response.json();
    },
    enabled: !!problemId,
  });

  return { submissions: submissions || [], loading: isLoading };
}

// SDK Types and interfaces for question type plugins
export interface QuestionTypeManifest {
  name: string;
  version: string;
  apiVersion: string;
  capabilities: {
    judgingMode: 'batch' | 'interactive' | 'output-only';
    supportsInteractive: boolean;
    supportsCustomCheckerUI: boolean;
  };
  entry: string;
  ui?: {
    problemExtraPanels?: string[];
    submissionAugment?: string;
  };
}

export interface QuestionTypePlugin {
  manifest: QuestionTypeManifest;
  component: React.ComponentType<QuestionTypeProps>;
}

export interface QuestionTypeProps {
  problem: Problem;
  onSubmit: (code: string, language: string) => void;
  submitting: boolean;
}

// Built-in question types
export const BUILTIN_QUESTION_TYPES: Record<string, QuestionTypeManifest> = {
  standard: {
    name: "Standard (IOI/ICPC)",
    version: "1.0.0",
    apiVersion: "1.x",
    capabilities: {
      judgingMode: "batch",
      supportsInteractive: false,
      supportsCustomCheckerUI: false,
    },
    entry: "./built-in/standard.js",
  },
  "output-only": {
    name: "Output-Only",
    version: "1.0.0", 
    apiVersion: "1.x",
    capabilities: {
      judgingMode: "output-only",
      supportsInteractive: false,
      supportsCustomCheckerUI: true,
    },
    entry: "./built-in/output-only.js",
  },
};
