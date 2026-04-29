import React from "react";
import { ErrorBoundary } from "./components/ErrorBoundary";

// Placeholder page components — replace with real implementations
const ProposalList = React.lazy(() => import("./pages/ProposalList"));
const ProposalDetail = React.lazy(() => import("./pages/ProposalDetail"));
const VotingPanel = React.lazy(() => import("./pages/VotingPanel"));

export default function App() {
  return (
    <ErrorBoundary section="App">
      <ErrorBoundary section="ProposalList">
        <React.Suspense fallback={<p>Loading…</p>}>
          <ProposalList />
        </React.Suspense>
      </ErrorBoundary>

      <ErrorBoundary section="ProposalDetail">
        <React.Suspense fallback={<p>Loading…</p>}>
          <ProposalDetail />
        </React.Suspense>
      </ErrorBoundary>

      <ErrorBoundary section="VotingPanel">
        <React.Suspense fallback={<p>Loading…</p>}>
          <VotingPanel />
        </React.Suspense>
      </ErrorBoundary>
    </ErrorBoundary>
  );
}
