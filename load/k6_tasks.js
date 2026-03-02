// k6 load test harness for the Tasks API tutorial service.
//
// Scenario:
// - Create tasks
// - List tasks with/without filters
// - Patch task completion state
// - Delete tasks
//
// Run example:
// k6 run load/k6_tasks.js
//
// Env overrides:
// BASE_URL=http://localhost:3000
// TASK_TITLE_PREFIX=LoadTask

import http from "k6/http";
import { check, sleep } from "k6";

const BASE_URL = __ENV.BASE_URL || "http://localhost:3000";
const TASK_TITLE_PREFIX = __ENV.TASK_TITLE_PREFIX || "LoadTask";

export const options = {
  stages: [
    { duration: "30s", target: 10 },
    { duration: "2m", target: 20 },
    { duration: "30s", target: 0 },
  ],
  thresholds: {
    http_req_failed: ["rate<0.01"],
    http_req_duration: ["p(95)<300"],
  },
};

function createTask(iteration) {
  const payload = JSON.stringify({
    title: `${TASK_TITLE_PREFIX}-${__VU}-${iteration}-${Date.now()}`,
  });

  const response = http.post(`${BASE_URL}/api/v1/tasks`, payload, {
    headers: { "Content-Type": "application/json" },
  });

  check(response, {
    "create: status is 201": (res) => res.status === 201,
  });

  if (response.status !== 201) {
    return null;
  }

  return response.json("id");
}

function listTasks() {
  const response = http.get(
    `${BASE_URL}/api/v1/tasks?limit=20&offset=0&completed=false&q=${encodeURIComponent(TASK_TITLE_PREFIX)}`,
  );

  check(response, {
    "list: status is 200": (res) => res.status === 200,
  });
}

function updateTask(taskId) {
  if (taskId === null || taskId === undefined) {
    return;
  }

  const payload = JSON.stringify({ completed: true });
  const response = http.patch(`${BASE_URL}/api/v1/tasks/${taskId}`, payload, {
    headers: { "Content-Type": "application/json" },
  });

  check(response, {
    "patch: status is 200": (res) => res.status === 200,
  });
}

function deleteTask(taskId) {
  if (taskId === null || taskId === undefined) {
    return;
  }

  const response = http.del(`${BASE_URL}/api/v1/tasks/${taskId}`);

  check(response, {
    "delete: status is 204": (res) => res.status === 204,
  });
}

export default function () {
  const taskId = createTask(__ITER);
  listTasks();
  updateTask(taskId);

  if (__ITER % 2 === 0) {
    deleteTask(taskId);
  }

  sleep(0.2);
}
