#!/usr/bin/env bun
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { JoyClient, configFromEnv } from "./client.ts";
import { buildServer } from "./server.ts";

const server = buildServer(new JoyClient(configFromEnv()));
await server.connect(new StdioServerTransport());
