import store from "@/store";
import axios, { AxiosInstance, AxiosResponse } from "axios";

export interface ICredentials {
  access_token: string;
  refresh_token: string;
}

export interface IGateList {
  id: number;
  name: string;
  description: string;
}

export default class Api {
  client: AxiosInstance;
  public accessToken?: string;
  public refreshToken?: string;
  refreshRequest?: Promise<AxiosResponse<ICredentials>>;

  constructor(options?: {
    accessToken?: string;
    refreshToken?: string;
    client?: AxiosInstance;
  }) {
    this.client = options?.client || axios.create();
    this.accessToken = options?.accessToken;
    this.refreshToken = options?.refreshToken;

    this.client.interceptors.request.use(
      (config) => {
        if (!this.accessToken) {
          return config;
        }

        const newConfig = {
          headers: {},
          ...config,
        };

        newConfig.headers.Authorization = `Bearer ${this.accessToken}`;
        return newConfig;
      },
      (e) => Promise.reject(e)
    );

    this.client.interceptors.response.use(
      (r) => r,
      async (error) => {
        if (
          !this.refreshToken ||
          error.response.status !== 401 ||
          error.config.retry
        ) {
          store.commit("logout");
          throw error;
        }

        if (!this.refreshRequest) {
          this.refreshRequest = this.client?.post("/auth/refresh", {
            refresh_token: this.refreshToken,
          });
        }
        const { data } = await this.refreshRequest;
        this.accessToken = data.access_token;
        this.refreshToken = data.refresh_token;

        if (this.accessToken && this.refreshRequest) {
          store.commit("login", this);
        }

        const newRequest = {
          ...error.config,
          retry: true,
        };

        return this.client(newRequest);
      }
    );
  }

  async login(login: string, password: string): Promise<void> {
    const response: AxiosResponse<ICredentials> = await this.client.post(
      "/auth/login",
      {
        login,
        password,
      }
    );

    this.accessToken = response.data.access_token;
    this.refreshToken = response.data.refresh_token;
  }

  async logout(): Promise<void> {
    await this.client.post("/auth/logout");

    this.accessToken = undefined;
    this.refreshToken = undefined;
  }

  async getGates(): Promise<Array<IGateList>> {
    const { data } = await this.client.get("/gates/list");

    return data.gates;
  }

  async openGate(gate: string): Promise<void> {
    await this.client.post(`/gates/open/${gate}`);
  }
}
