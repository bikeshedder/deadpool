CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public;

CREATE TABLE event (
    id uuid DEFAULT public.gen_random_uuid() NOT NULL,
    title text NOT NULL
);

COPY event (id, title) FROM stdin;
bec734be-97bd-4f48-b042-5ee859998c56	event1
d36bc40a-fac4-4194-bec4-9ba8c602d994	event2
18012fcf-3c53-4518-b9a6-22479ebe0212	event3
6c4094fc-f8f7-4869-9c57-042bc43f257d	event4
e6173a6c-7cf4-4bc5-b676-0871d2705ed6	event5
0d246195-537f-42dd-b7b3-6495116b8f56	event6
c6b452b9-c1f3-4e11-9237-e22e0d6f14bd	event7
33ddb912-5bb1-4ba6-a03c-31a087ca8992	event8
0da79dbe-63e9-4a0a-a3cb-34cfc451aa7e	event9
dec2f1cd-01af-41ed-a0d9-83a34a1c7b6d	event10
ecc1e7e4-828c-4ca3-a8a1-07444bb91300	event11
c6ca56b1-9e02-4381-a248-3b9ef099eb93	event12
f94693f0-789a-4a1f-a218-7399c294f00a	event13
c54ee1d7-2f8c-41c5-9478-a01f4c1c0e0b	event14
aa213e86-43fe-448d-8862-f2af9c686c82	event15
7135cb1f-00ee-47be-b212-d45a791082ea	event16
b7dfa22b-f3a2-4547-a5df-2a6d3f54c90f	event17
c339d9fd-ce35-4578-a7c5-908cf619a321	event18
7c49ba99-fdc3-4618-b23b-89ea3e5c4ee4	event19
499b5807-abd2-4c24-9faa-5f463e6a19e3	event20
d2ccbdb5-5b1e-4113-aded-24634ba1fb03	event21
f2c82fc6-5b9a-446a-a6bc-27e814b1521b	event22
a818fbed-93dd-4e44-84a8-3591dbff3349	event23
56dd7210-3869-4f1f-9f7a-fc58b3265f63	event24
5803b0e8-98fe-47cd-b61d-478949b2ad70	event25
b4a8d7da-ee5f-4d95-b452-86400697d051	event26
f55fd18a-5532-453c-990e-098df71f2c24	event27
3938cd54-65cf-4638-aba8-5861d9ba729e	event28
\.
