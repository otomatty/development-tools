/**
 * Project Dashboard Page
 *
 * Project detail page showing project information and issues.
 * This is a placeholder - actual implementation will be done in Phase 3.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/135
 */

import { useParams } from 'react-router-dom';

const ProjectDashboard = () => {
  const { id } = useParams<{ id: string }>();

  return (
    <div className="min-h-screen bg-dt-bg text-dt-text p-8">
      <h1 className="text-4xl font-bold text-gm-accent-cyan">Project Dashboard</h1>
      <p className="mt-4 text-dt-text-sub">Project ID: {id}</p>
      <p className="mt-2 text-dt-text-sub">Coming soon...</p>
    </div>
  );
};

export default ProjectDashboard;
