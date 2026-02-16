// Клиент API МойСклад

import type {
  Meta,
  EntityRef,
  Product,
  StockRow,
  ProcessingPlan,
  Processing,
  Demand,
  ApiResponse,
} from './types';

const MOYSKLAD_API_BASE = 'https://api.moysklad.ru/api/remap/1.2';

export class MoyskladClient {
  private token: string;

  constructor(token: string) {
    this.token = token;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = endpoint.startsWith('http') ? endpoint : `${MOYSKLAD_API_BASE}${endpoint}`;

    const response = await fetch(url, {
      ...options,
      headers: {
        'Authorization': `Bearer ${this.token}`,
        'Accept-Encoding': 'gzip',
        'Content-Type': 'application/json',
        ...options.headers,
      },
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`Moysklad API Error ${response.status}: ${errorText}`);
    }

    return response.json();
  }

  // Получить список складов
  async getStores(): Promise<ApiResponse<EntityRef>> {
    return this.request<ApiResponse<EntityRef>>('/entity/store');
  }

  // Найти склад по названию
  async findStoreByName(name: string): Promise<EntityRef | null> {
    const response = await this.request<ApiResponse<EntityRef>>(
      `/entity/store?filter=name=${encodeURIComponent(name)}`
    );
    return response.rows?.[0] || null;
  }

  // Получить остатки по складу
  async getStockByStore(storeId: string): Promise<StockRow[]> {
    const response = await this.request<{ rows: StockRow[] }>(
      `/report/stock/all?filter=stockStore=${storeId}&limit=1000`
    );
    return response.rows || [];
  }

  // Получить остаток конкретного товара
  async getProductStock(productId: string, storeId: string): Promise<number> {
    const response = await this.request<{ rows: StockRow[] }>(
      `/report/stock/all?filter=assortmentId=${productId};stockStore=${storeId}`
    );
    
    if (response.rows && response.rows.length > 0) {
      const row = response.rows[0];
      // Возвращаем доступный остаток (минус резерв)
      return (row.stock || 0) - (row.reserve || 0);
    }
    return 0;
  }

  // Получить товар по ID с атрибутами
  async getProduct(productId: string): Promise<Product> {
    return this.request<Product>(`/entity/product/${productId}?expand=attributes`);
  }

  // Получить ассортимент по ID (для получения атрибутов)
  async getAssortmentItem(assortmentId: string): Promise<Product> {
    return this.request<Product>(`/entity/assortment/${assortmentId}?expand=attributes`);
  }

  // Получить тех. карту по названию
  async findProcessingPlanByName(name: string): Promise<ProcessingPlan | null> {
    const response = await this.request<ApiResponse<ProcessingPlan>>(
      `/entity/processingplan?filter=name=${encodeURIComponent(name)}`
    );
    return response.rows?.[0] || null;
  }

  // Получить тех. карту по ID
  async getProcessingPlan(planId: string): Promise<ProcessingPlan> {
    return this.request<ProcessingPlan>(
      `/entity/processingplan/${planId}?expand=materials,products`
    );
  }

  // Создать тех. операцию
  async createProcessing(data: {
    processingPlan: EntityRef;
    store: EntityRef;
    organization: EntityRef;
    products: Array<{
      product: EntityRef;
      quantity: number;
      processingPlanPosition?: { meta: Meta; id: string };
    }>;
    materials: Array<{
      product: EntityRef;
      quantity: number;
      processingPlanPosition?: { meta: Meta; id: string };
    }>;
    name?: string;
    description?: string;
  }): Promise<Processing> {
    const processingData = {
      processingPlan: {
        meta: data.processingPlan.meta,
      },
      store: {
        meta: data.store.meta,
      },
      organization: {
        meta: data.organization.meta,
      },
      products: data.products.map((p) => ({
        product: {
          meta: p.product.meta,
        },
        quantity: p.quantity,
        ...(p.processingPlanPosition && {
          processingPlanPosition: p.processingPlanPosition,
        }),
      })),
      materials: data.materials.map((m) => ({
        product: {
          meta: m.product.meta,
        },
        quantity: m.quantity,
        ...(m.processingPlanPosition && {
          processingPlanPosition: m.processingPlanPosition,
        }),
      })),
      ...(data.name && { name: data.name }),
      ...(data.description && { description: data.description }),
    };

    return this.request<Processing>('/entity/processing', {
      method: 'POST',
      body: JSON.stringify(processingData),
    });
  }

  // Провести тех. операцию
  async applyProcessing(processingId: string): Promise<Processing> {
    return this.request<Processing>(`/entity/processing/${processingId}`, {
      method: 'PUT',
      body: JSON.stringify({ applicable: true }),
    });
  }

  // Получить организацию (первую из списка)
  async getOrganization(): Promise<EntityRef | null> {
    const response = await this.request<ApiResponse<EntityRef>>('/entity/organization');
    return response.rows?.[0] || null;
  }

  // Получить отгрузку по ID
  async getDemand(demandId: string): Promise<Demand> {
    return this.request<Demand>(
      `/entity/demand/${demandId}?expand=positions,store,organization,agent`
    );
  }

  // Получить список доп. полей для товаров
  async getProductAttributes(): Promise<ApiResponse<{ id: string; name: string; type: string }>> {
    return this.request<ApiResponse<{ id: string; name: string; type: string }>>(
      '/entity/product/metadata/attributes'
    );
  }

  // Найти доп. поле по названию
  async findAttributeByName(name: string): Promise<{ id: string; name: string; type: string } | null> {
    const response = await this.getProductAttributes();
    return response.rows?.find((attr) => attr.name === name) || null;
  }
}

// Создаём экземпляр клиента
export function createMoyskladClient(token: string): MoyskladClient {
  return new MoyskladClient(token);
}
